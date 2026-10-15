const { ethers } = require("hardhat");
const { expect } = require("chai");
const { expectRevert } = require("./helpers/utils");

describe("DeployFactory (CREATE3)", function () {
    let factory, deployer, attacker;
    let nodeCode; // creation code of a small contract: MockLightNode
    const salt = (s) => ethers.utils.keccak256(ethers.utils.toUtf8Bytes(s));

    before(async () => {
        [deployer, attacker] = await ethers.getSigners();
        nodeCode = (await ethers.getContractFactory("MockLightNode")).bytecode;
    });

    beforeEach(async () => {
        factory = await (await ethers.getContractFactory("DeployFactory")).deploy();
        await factory.deployed();
    });

    it("deploys to the address returned by getAddress and emits contractAddress", async () => {
        const s = salt("a");
        const predicted = await factory.getAddress(s);
        await expect(factory.deploy(s, nodeCode, 0)).to.emit(factory, "contractAddress").withArgs(predicted);
        expect(await ethers.provider.getCode(predicted)).to.not.equal("0x");
        const node = (await ethers.getContractFactory("MockLightNode")).attach(predicted);
        expect(await node.nodeType()).to.equal(1);
    });

    it("the address depends on the salt only: not on the code and not on the caller", async () => {
        const s = salt("same");
        const a1 = await factory.getAddress(s);
        const a2 = await factory.connect(attacker).getAddress(s);
        expect(a1).to.equal(a2);
        await factory.connect(attacker).deploy(s, nodeCode, 0);
        expect(await ethers.provider.getCode(a1)).to.not.equal("0x");
    });

    it("different salts give different addresses", async () => {
        expect(await factory.getAddress(salt("1"))).to.not.equal(await factory.getAddress(salt("2")));
    });

    it("different factories give different addresses for the same salt", async () => {
        const f2 = await (await ethers.getContractFactory("DeployFactory")).deploy();
        expect(await factory.getAddress(salt("x"))).to.not.equal(await f2.getAddress(salt("x")));
    });

    it("a salt can only be used once", async () => {
        const s = salt("once");
        await factory.deploy(s, nodeCode, 0);
        await expectRevert(factory.deploy(s, nodeCode, 0), "DEPLOYMENT_FAILED");
    });

    it("reverts when the constructor reverts or no runtime code is produced", async () => {
        await expectRevert(factory.deploy(salt("e1"), "0xfe", 0), "INITIALIZATION_FAILED"); // invalid opcode
        await expectRevert(factory.deploy(salt("e2"), "0x", 0), "INITIALIZATION_FAILED"); // empty creation code
    });

    it("[known issue] deploy is not payable and the factory holds no ether, so value > 0 always fails", async () => {
        await expectRevert(factory.deploy(salt("v"), nodeCode, 1), "INITIALIZATION_FAILED");
        await expectRevert(factory.deploy(salt("v"), nodeCode, 0, { value: 1 }));
    });

    it("[known issue] the salt is not bound to the caller: anyone can front-run and squat an address", async () => {
        const s = salt("victim-contract-v1");
        const predicted = await factory.getAddress(s);
        // attacker sees the salt (it is published for cross chain deployments) and deploys other code first
        await factory.connect(attacker).deploy(s, "0x600160005360016000f3", 0); // tiny contract returning 1 byte of code
        const code = await ethers.provider.getCode(predicted);
        expect(code).to.not.equal("0x");
        // the intended deployment now fails and the address is permanently occupied by the attacker's code
        await expectRevert(factory.deploy(s, nodeCode, 0), "DEPLOYMENT_FAILED");
    });
});

describe("Role", function () {
    let role, admin, m1, m2;
    const MANAGER = ethers.utils.id("MANAGER_ROLE");
    const ADMIN = ethers.constants.HashZero;

    before(async () => {
        [admin, m1, m2] = await ethers.getSigners();
    });

    beforeEach(async () => {
        role = await (await ethers.getContractFactory("Role")).deploy();
    });

    it("deployer is admin and manager", async () => {
        expect(await role.hasRole(ADMIN, admin.address)).to.equal(true);
        expect(await role.hasRole(MANAGER, admin.address)).to.equal(true);
        expect(await role.MANAGER_ROLE()).to.equal(MANAGER);
    });

    it("only the admin can add / remove managers", async () => {
        await expectRevert(role.connect(m1).addManager(m1.address), "AccessControl");
        await expectRevert(role.connect(m1).removeManager(admin.address), "AccessControl");
        await role.addManager(m1.address);
        expect(await role.hasRole(MANAGER, m1.address)).to.equal(true);
        await role.removeManager(m1.address);
        expect(await role.hasRole(MANAGER, m1.address)).to.equal(false);
    });

    it("a manager is not an admin and cannot add other managers", async () => {
        await role.addManager(m1.address);
        await expectRevert(role.connect(m1).addManager(m2.address), "AccessControl");
    });

    it("adding the same manager twice is harmless; removing a non-manager is a no-op", async () => {
        await role.addManager(m1.address);
        await role.addManager(m1.address);
        await role.removeManager(m2.address);
        expect(await role.hasRole(MANAGER, m1.address)).to.equal(true);
        expect(await role.hasRole(MANAGER, m2.address)).to.equal(false);
    });
});
