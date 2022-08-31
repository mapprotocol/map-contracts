import { time, loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
let data = require("../data/blocks.js");
import {
    BlockHeader, getBlock,
    TxLog, ReceiptProof,
    TxReceipt, index2key,
    ProofData, DProofData
} from "../utils/Util"


let uri = process.env.BSCURI;

let chainId = process.env.CHAINID;

let epochNum = 200;
let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen
describe("LightNode", function () {
    // We define a fixture to reuse the same setup in every test.
    // We use loadFixture to run this setup once, snapshot that state,
    // and reset Hardhat Network to that snapshopt in every test.
    async function deployFixture() {
        let [wallet] = await ethers.getSigners();

        const LightNode = await ethers.getContractFactory("LightNode");

        const lightNode = await LightNode.deploy(chainId, minEpochBlockExtraDataLen);

        await lightNode.connect(wallet).deployed();

        const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

        let initData = LightNode.interface.encodeFunctionData("initialize", [chainId, minEpochBlockExtraDataLen]);

        const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

        await lightNodeProxy.connect(wallet).deployed();

        let proxy = LightNode.attach(lightNodeProxy.address);

        return proxy;

    }

    describe("Deployment", function () {

        it("initBlock must owner", async function () {


            let [wallet, other] = await ethers.getSigners();

            const lightNode = await loadFixture(deployFixture);


            let preValidators = '0x' + getValidators(data.block20853000.extraData);

            await expect(lightNode.connect(other).initBlock(preValidators, data.block20853200)).to.be.revertedWith('lightnode :: only admin')

        });

        it("initBlock ok", async function () {


            let [wallet, other] = await ethers.getSigners();

            const lightNode = await loadFixture(deployFixture);


            let preValidators = '0x' + getValidators(data.block20853000.extraData);

            await lightNode.connect(wallet).initBlock(preValidators, data.block20853200);

            let current = await lightNode.headerHeight();

            expect(current).to.eq(20853200)

        });

        it("Implementation upgradle must admin", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy(chainId, minEpochBlockExtraDataLen);
            await newImplement.deployed();

            await expect(lightNode.connect(other).upgradeTo(newImplement.address)).to.be.revertedWith('LightNode: only Admin can upgrade');

        });


        it("Implementation upgradle ok", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy(chainId, minEpochBlockExtraDataLen);
            await newImplement.deployed();

            let oldImplement = await lightNode.getImplementation();

            expect(oldImplement).to.not.eq(newImplement.address);

            await lightNode.connect(wallet).upgradeTo(newImplement.address);

            expect(await lightNode.getImplementation()).to.eq(newImplement.address);

        });


        it("change admin ", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.eq(wallet.address);

            await expect(lightNode.connect(other).changeAdmin(other.address)).to.be.revertedWith("lightnode :: only admin");

            await expect(lightNode.connect(wallet).changeAdmin(ethers.constants.AddressZero)).to.be.revertedWith("zero address");

            await lightNode.connect(wallet).changeAdmin(other.address);

            expect(await lightNode.getAdmin()).to.eq(other.address);

        });


        it("trigglePause  only admin ", async function () {

            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

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

            let lightNode = await loadFixture(deployFixture);


            await lightNode.connect(wallet).trigglePause(true);

            let preValidators = '0x' + getValidators(data.block20853000.extraData);

            await lightNode.connect(wallet).initBlock(preValidators, data.block20853200);

            await expect(lightNode.updateBlockHeader(data.addBlocks)).to.be.revertedWith('Pausable: paused');

        });

        it("updateBlockHeader ... ok ", async function () {

            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let preValidators = '0x' + getValidators(data.block20853000.extraData);

            await lightNode.connect(wallet).initBlock(preValidators, data.block20853200);

            await lightNode.updateBlockHeader(data.addBlocks);

            let current = await lightNode.headerHeight();

            expect(current).to.eq(20853201)

        });

        it("verifyProofData ... unconfirm block ", async function () {

            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let preValidators = '0x' + getValidators(data.block20853000.extraData);

            await lightNode.connect(wallet).initBlock(preValidators, data.block20853200);

            let receiptProof = new ReceiptProof(data.txReceipt, index2key(BigNumber.from(data.proof.key).toNumber(), data.proof.proof.length), data.proof.proof);

            let proofData = new ProofData(BigNumber.from(data.proof.key), receiptProof);

            let proofBytes = await lightNode.getBytes(proofData);

            let result = await lightNode.verifyProofData(proofBytes, { gasLimit: 20000000 });

            expect("unconfirm block").to.eq(result.message);
        });


        it("verifyProofData ... ok ", async function () {

            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let preValidators = '0x' + getValidators(data.block20853000.extraData);

            await lightNode.connect(wallet).initBlock(preValidators, data.block20853200);

            await lightNode.updateBlockHeader(data.addBlocks);

            let current = await lightNode.headerHeight();

            expect(current).to.eq(20853201)

            let receiptProof = new ReceiptProof(data.txReceipt, index2key(BigNumber.from(data.proof.key).toNumber(), data.proof.proof.length), data.proof.proof);

            let proofData = new ProofData(BigNumber.from(20853201), receiptProof);

            let proofBytes = await lightNode.getBytes(proofData);

            let result = await lightNode.verifyProofData(proofBytes, { gasLimit: 20000000 });

            expect(result.success).to.true;
        });



    });




});

function getValidators(extraData: string) {
    let length = extraData.length;
    return extraData.substring(66, length - 130);
}