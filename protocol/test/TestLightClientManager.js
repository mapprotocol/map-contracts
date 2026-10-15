const { ethers } = require("hardhat");
const { expect } = require("chai");
const { expectRevert, deployProxy } = require("./helpers/utils");

describe("LightClientManager", function () {
    const CHAIN = 97;
    const NOT_REG = "manager: not register";
    let owner, other;
    let mgr, node;

    before(async () => {
        [owner, other] = await ethers.getSigners();
    });

    beforeEach(async () => {
        const init = new ethers.utils.Interface(["function initialize()"]).encodeFunctionData("initialize");
        ({ contract: mgr } = await deployProxy("LightClientManager", "LightClientManagerProxy", init));
        node = await (await ethers.getContractFactory("MockLightNode")).deploy();
        await node.deployed();
    });

    describe("admin", () => {
        it("deployer is admin and cannot re-initialize", async () => {
            expect(await mgr.getAdmin()).to.equal(owner.address);
            await expectRevert(mgr.initialize(), "already initialized");
        });

        it("proxy constructor rejects zero logic address", async () => {
            const Proxy = await ethers.getContractFactory("LightClientManagerProxy");
            await expectRevert(Proxy.deploy(ethers.constants.AddressZero, "0x"));
        });

        it("changeAdmin: admin only and non-zero", async () => {
            await expectRevert(mgr.connect(other).changeAdmin(other.address), "only owner");
            await expectRevert(mgr.changeAdmin(ethers.constants.AddressZero), "address is zero");
            await mgr.changeAdmin(other.address);
            expect(await mgr.getAdmin()).to.equal(other.address);
            await expectRevert(mgr.register(CHAIN, node.address), "only owner");
        });

        it("upgrade is admin only and works", async () => {
            const impl2 = await (await ethers.getContractFactory("LightClientManager")).deploy();
            await expectRevert(mgr.connect(other).upgradeTo(impl2.address), "only Admin can upgrade");
            await mgr.register(CHAIN, node.address);
            await mgr.upgradeTo(impl2.address);
            expect(await mgr.getImplementation()).to.equal(impl2.address);
            expect(await mgr.lightClientContract(CHAIN)).to.equal(node.address); // storage kept
        });

        it("the bare implementation is locked and cannot be initialized", async () => {
            const impl = await (await ethers.getContractFactory("LightClientManager")).deploy();
            await expectRevert(impl.connect(other).initialize(), "already initialized");
        });
    });

    describe("register", () => {
        it("admin only", async () => {
            await expectRevert(mgr.connect(other).register(CHAIN, node.address), "only owner");
        });

        it("stores the client and emits LightClientRegister", async () => {
            await expect(mgr.register(CHAIN, node.address))
                .to.emit(mgr, "LightClientRegister")
                .withArgs(CHAIN, node.address);
            expect(await mgr.lightClientContract(CHAIN)).to.equal(node.address);
        });

        it("can be overwritten with a new client", async () => {
            const node2 = await (await ethers.getContractFactory("MockLightNode")).deploy();
            await mgr.register(CHAIN, node.address);
            await mgr.register(CHAIN, node2.address);
            await mgr.updateBlockHeader(CHAIN, "0x01");
            expect(await node2.lastBlockHeader()).to.equal("0x01");
            expect(await node.lastBlockHeader()).to.equal("0x");
        });

        it("[known issue] accepts address(0) and then silently un-registers the chain", async () => {
            await mgr.register(CHAIN, node.address);
            await mgr.register(CHAIN, ethers.constants.AddressZero);
            await expectRevert(mgr.nodeType(CHAIN), NOT_REG);
        });
    });

    describe("unregistered chain", () => {
        it("every entry point reverts with 'not register'", async () => {
            const calls = [
                () => mgr.updateBlockHeader(CHAIN, "0x"),
                () => mgr.updateLightClient(CHAIN, "0x"),
                () => mgr.notifyLightClient(CHAIN, other.address, "0x"),
                () => mgr.callStatic.verifyProofDataWithCache(CHAIN, "0x"),
                () => mgr.verifyProofData(CHAIN, "0x"),
                () => mgr.clientState(CHAIN),
                () => mgr.headerHeight(CHAIN),
                () => mgr.verifiableHeaderRange(CHAIN),
                () => mgr.finalizedState(CHAIN, "0x"),
                () => mgr.isVerifiable(CHAIN, 1, ethers.constants.HashZero),
                () => mgr.nodeType(CHAIN),
            ];
            for (const c of calls) await expectRevert(c(), NOT_REG);
        });
    });

    describe("forwarding to the registered client", () => {
        beforeEach(async () => {
            await mgr.register(CHAIN, node.address);
        });

        it("updateBlockHeader forwards data and the caller seen by the node is the manager", async () => {
            await mgr.connect(other).updateBlockHeader(CHAIN, "0xdeadbeef");
            expect(await node.lastBlockHeader()).to.equal("0xdeadbeef");
            expect(await node.lastCaller()).to.equal(mgr.address);
        });

        it("updateLightClient forwards data", async () => {
            await mgr.updateLightClient(CHAIN, "0x0102");
            expect(await node.lastUpdateData()).to.equal("0x0102");
        });

        it("bubbles up reverts of the client", async () => {
            await node.setFail(true);
            await expectRevert(mgr.updateBlockHeader(CHAIN, "0x01"), "mock: failed");
            await expectRevert(mgr.updateLightClient(CHAIN, "0x01"), "mock: failed");
            await expectRevert(mgr.verifyProofData(CHAIN, "0x01"), "mock: failed");
            await expectRevert(mgr.verifyProofDataWithCache(CHAIN, "0x01"), "mock: failed");
        });

        it("verifyProofDataWithCache is stateful and returns the client result", async () => {
            const r = await mgr.callStatic.verifyProofDataWithCache(CHAIN, "0xaabb");
            expect(r.success).to.equal(true);
            expect(r.message).to.equal("");
            expect(r.logs).to.equal("0xaabb");
            await mgr.verifyProofDataWithCache(CHAIN, "0xaabb");
            expect(await node.cacheCalls()).to.equal(1);
        });

        it("verifyProofData returns the client result", async () => {
            const r = await mgr.verifyProofData(CHAIN, "0x1234");
            expect(r.success).to.equal(true);
            expect(r.logs).to.equal("0x1234");
        });

        it("view helpers are proxied", async () => {
            expect(await mgr.clientState(CHAIN)).to.equal("0xc0ffee");
            expect(await mgr.headerHeight(CHAIN)).to.equal(100);
            const range = await mgr.verifiableHeaderRange(CHAIN);
            expect(range[0]).to.equal(10);
            expect(range[1]).to.equal(100);
            expect(await mgr.finalizedState(CHAIN, "0x01")).to.equal(
                ethers.utils.hexlify(ethers.utils.concat([ethers.utils.toUtf8Bytes("finalized:"), "0x01"]))
            );
            expect(await mgr.isVerifiable(CHAIN, 50, ethers.constants.HashZero)).to.equal(true);
            expect(await mgr.isVerifiable(CHAIN, 5, ethers.constants.HashZero)).to.equal(false);
            expect(await mgr.isVerifiable(CHAIN, 101, ethers.constants.HashZero)).to.equal(false);
            expect(await mgr.nodeType(CHAIN)).to.equal(1);
            await node.setNodeType(4);
            expect(await mgr.nodeType(CHAIN)).to.equal(4);
        });

        it("chains are isolated from each other", async () => {
            const node2 = await (await ethers.getContractFactory("MockLightNode")).deploy();
            await node2.setNodeType(3);
            await mgr.register(CHAIN + 1, node2.address);
            expect(await mgr.nodeType(CHAIN)).to.equal(1);
            expect(await mgr.nodeType(CHAIN + 1)).to.equal(3);
        });

        it("notifyLightClient only emits ManagerNotifySend (client is not called)", async () => {
            const tx = await mgr.connect(other).notifyLightClient(CHAIN, other.address, "0xabcd");
            const rc = await tx.wait();
            await expect(tx).to.emit(mgr, "ManagerNotifySend").withArgs(CHAIN, other.address, rc.blockNumber, "0xabcd");
        });

        it("[known issue] notifyLightClient is permissionless and `_from` is spoofable", async () => {
            const tx = await mgr.connect(other).notifyLightClient(CHAIN, owner.address, "0x");
            const rc = await tx.wait();
            await expect(tx).to.emit(mgr, "ManagerNotifySend").withArgs(CHAIN, owner.address, rc.blockNumber, "0x");
        });
    });
});
