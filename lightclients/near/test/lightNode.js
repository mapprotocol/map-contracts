const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");
const { expect } = require("chai");
const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require("./utils/borsh");
const hre = require("hardhat");
const bs58 = require("bs58");
const { ethers } = require("hardhat");

describe("lightNode", function () {
    async function deployLightNode() {
        let [wallet] = await hre.ethers.getSigners();
        const LightNode = await hre.ethers.getContractFactory("LightNode");
        const lightNode = await LightNode.connect(wallet).deploy();
        await lightNode.deployed();

        const LightNodeProxy = await hre.ethers.getContractFactory("LightNodeProxy");

        const iface = new hre.ethers.utils.Interface([
            "function initialize(address _owner, bytes[2] memory initDatas)",
        ]);

        let block = "0x" + borshify(require("./data/block.json")).toString("hex");
        let validators = "0x" + borshifyInitialValidators(require("./data/validators.json").next_bps).toString("hex");
        let arr = [validators, block];
        let data = iface.encodeFunctionData("initialize", [wallet.address, arr]);
        const lightNodeProxy = await LightNodeProxy.connect(wallet).deploy(lightNode.address, data);
        await lightNodeProxy.deployed();
        const proxy = LightNode.attach(lightNodeProxy.address);
        return proxy;
    }

    describe("Deployment", function () {
        it("init correct", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let nextEpochId = await lightNode.nextEpochId();

            expect(nextEpochId).to.eq(
                "0x" + bs58.decode("8xm4iXSc4mxNgArgUVLmQDMM1QthEaijdMu98CUd4Mqf").toString("hex")
            );
        });

        it("Implementation upgradle must admin", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await hre.ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            await expect(lightNode.connect(other).upgradeTo(newImplement.address)).to.be.revertedWith(
                "LightNode: only Admin can upgrade"
            );
        });

        it("Implementation upgradle ok", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await hre.ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            let oldImplement = await lightNode.getImplementation();

            expect(oldImplement).to.not.eq(newImplement.address);

            await lightNode.connect(wallet).upgradeTo(newImplement.address);

            expect(await lightNode.getImplementation()).to.eq(newImplement.address);
        });

        it("change admin ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let admin = await lightNode.getAdmin();

            expect(admin).to.eq(wallet.address);

            await expect(lightNode.connect(other).setPendingAdmin(other.address)).to.be.revertedWith(
                "lightnode :: only admin"
            );

            await expect(lightNode.connect(wallet).setPendingAdmin(ethers.constants.AddressZero)).to.be.revertedWith(
                "Ownable: pendingAdmin is the zero address"
            );

            await (await lightNode.connect(wallet).setPendingAdmin(other.address)).wait();

            let pendingAdmin = await lightNode.pendingAdmin();

            expect(pendingAdmin).eq(other.address);

            await expect(lightNode.connect(wallet).changeAdmin()).to.be.revertedWith("only pendingAdmin");

            await (await lightNode.connect(other).changeAdmin()).wait();

            expect(await lightNode.getAdmin()).to.eq(other.address);
        });

        it("togglePause  only admin ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let paused = await lightNode.paused();

            expect(paused).to.false;

            await expect(lightNode.connect(other).togglePause(true)).to.be.revertedWith("lightnode :: only admin");

            await lightNode.connect(wallet).togglePause(true);

            expect(await lightNode.paused()).to.true;

            await lightNode.connect(wallet).togglePause(false);

            expect(await lightNode.paused()).to.false;
        });

        it("updateBlockHeader ... paused ", async function () {
            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            expect(await lightNode.curHeight()).to.be.equal(96136659);

            await lightNode.connect(wallet).togglePause(true);

            await expect(
                lightNode.connect(wallet).updateBlockHeader(borshify(require("./data/block.json")))
            ).to.be.revertedWith("Pausable: paused");
        });
    });
});
