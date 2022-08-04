const {
    loadFixture,
} = require("@nomicfoundation/hardhat-network-helpers");
const { expect } = require("chai");
const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('./utils/borsh');
const hre = require("hardhat");
const bs58 = require('bs58');
const { ethers } = require("hardhat");
const { promisify } = require('util');


let initData = '0x8129fc1c';

describe("lightNode", function () {

    async function deployLightNode() {
        let [wallet] = await hre.ethers.getSigners();

        const LightNode = await hre.ethers.getContractFactory("LightNode");
        const lightNode = await LightNode.connect(wallet).deploy();
        await lightNode.deployed();

        const LightNodeProxy = await hre.ethers.getContractFactory("LightNodeProxy");
        const lightNodeProxy = await LightNodeProxy.connect(wallet).deploy(lightNode.address, initData);
        await lightNodeProxy.deployed();
        const proxy = LightNode.attach(lightNodeProxy.address);
        return proxy
    }

    describe("Deployment", function () {


        it("initWithValidators must owner", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let block = borshify(require('./data/block.json'));

            let validators = borshifyInitialValidators(require('./data/validators.json').next_bps);

            await expect(lightNode.connect(other).initWithValidators(validators)).to.be.reverted;

        });


        it("initWithBlock must owner", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let block = borshify(require('./data/block.json'));

            let validators = borshifyInitialValidators(require('./data/validators.json').next_bps);

            await lightNode.connect(wallet).initWithValidators(validators);

            await expect(lightNode.connect(other).initWithBlock(block)).to.be.reverted;

        });

        it("init should be ok", async function () {

            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let block = borshify(require('./data/block.json'));

            let validators = borshifyInitialValidators(require('./data/validators.json').next_bps);

            await lightNode.connect(wallet).initWithValidators(validators);

            await lightNode.connect(wallet).initWithBlock(block);

            expect(await lightNode.curHeight()).to.be.equal(96136659);

        });

        it("Implementation upgradle must admin", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await hre.ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            await expect(lightNode.connect(other).upgradeTo(newImplement.address)).to.be.revertedWith('LightNode: only Admin can upgrade');

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

            await expect(lightNode.connect(other).changeAdmin(other.address)).to.be.revertedWith("lightnode :: only admin");

            await expect(lightNode.connect(wallet).changeAdmin(ethers.constants.AddressZero)).to.be.revertedWith("zero address");

            await lightNode.connect(wallet).changeAdmin(other.address);

            expect(await lightNode.getAdmin()).to.eq(other.address);

        });


        it("trigglePause  only admin ", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let paused = await lightNode.paused();

            expect(paused).to.false;

            await expect(lightNode.connect(other).trigglePause(true)).to.be.revertedWith("lightnode :: only admin");

            await lightNode.connect(wallet).trigglePause(true);

            expect(await lightNode.paused()).to.true;

            await lightNode.connect(wallet).trigglePause(false);

            expect(await lightNode.paused()).to.false;

        });


        it("updateBlockHeader ... paused ", async function () {

            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployLightNode);

            let block = borshify(require('./data/block.json'));

            let validators = borshifyInitialValidators(require('./data/validators.json').next_bps);

            await lightNode.connect(wallet).initWithValidators(validators);

            await lightNode.connect(wallet).initWithBlock(block);

            expect(await lightNode.curHeight()).to.be.equal(96136659);

            await lightNode.connect(wallet).trigglePause(true);

            await expect(lightNode.connect(wallet).updateBlockHeader(borshify(require('./data/block.json')))).to.be.revertedWith("Pausable: paused")


        });

    });
});
