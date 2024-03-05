import { time, loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
let data = require("../data/blocks.js");
import { BlockHeader, getBlock, TxLog, ReceiptProof, TxReceipt, index2key, ProofData } from "../utils/Util";

let chainId = 2206132;

describe("LightNode", function () {
    // We define a fixture to reuse the same setup in every test.
    // We use loadFixture to run this setup once, snapshot that state,
    // and reset Hardhat Network to that snapshot in every test.
    async function deployFixture() {
        let [wallet] = await ethers.getSigners();

        const MPTVerify = await ethers.getContractFactory("MPTVerify");

        const mPTVerify = await MPTVerify.deploy();

        await mPTVerify.connect(wallet).deployed();

        const LightNode = await ethers.getContractFactory("LightNode");
        const lightNode = await LightNode.deploy();

        await lightNode.connect(wallet).deployed();

        const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

        let initBlock = data.initBlock;

        let validators = data.init_validators;

        let initData = LightNode.interface.encodeFunctionData("initialize", [
            chainId,
            wallet.address,
            mPTVerify.address,
            initBlock,
            validators,
        ]);

        const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

        await lightNodeProxy.connect(wallet).deployed();

        let proxy = LightNode.attach(lightNodeProxy.address);

        return proxy;
    }

    describe("Deployment", function () {
        it("initBlock() -> correct", async function () {
            let [wallet, other] = await ethers.getSigners();

            const lightNode = await loadFixture(deployFixture);

            let current = await lightNode.headerHeight();

            expect(current).to.eq(21728760);
        });

        it("upgradeTo() -> reverts only Admin", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            await expect(lightNode.connect(other).upgradeTo(newImplement.address)).to.be.revertedWith(
                "LightNode: only Admin can upgrade"
            );
        });

        it("upgradeTo() -> correct", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.not.eq(other.address);

            const LightNode = await ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            let oldImplement = await lightNode.getImplementation();

            expect(oldImplement).to.not.eq(newImplement.address);

            await lightNode.connect(wallet).upgradeTo(newImplement.address);

            expect(await lightNode.getImplementation()).to.eq(newImplement.address);
        });

        it("changeAdmin() -> reverts only Admin", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.eq(wallet.address);

            await expect(lightNode.connect(other).setPendingAdmin(other.address)).to.be.revertedWith(
                "lightnode :: only admin"
            );
        });

        it("changeAdmin() -> reverts for zero address", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.eq(wallet.address);

            await expect(lightNode.connect(wallet).setPendingAdmin(ethers.constants.AddressZero)).to.be.revertedWith(
                "Ownable: pendingAdmin is the zero address"
            );
        });

        it("changeAdmin() -> correct ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();

            expect(admin).to.eq(wallet.address);

            await (await lightNode.connect(wallet).setPendingAdmin(other.address)).wait();

            let pendingAdmin = await lightNode.pendingAdmin();

            expect(pendingAdmin).eq(other.address);

            await expect(lightNode.connect(wallet).changeAdmin()).to.be.revertedWith("only pendingAdmin");

            await (await lightNode.connect(other).changeAdmin()).wait();

            expect(await lightNode.getAdmin()).to.eq(other.address);
        });

        it("togglePause() -> reverts  only admin ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let paused = await lightNode.paused();

            expect(paused).to.false;

            await expect(lightNode.connect(other).togglePause(true)).to.be.revertedWith("lightnode :: only admin");
        });

        it("togglePause() -> correct ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            let paused = await lightNode.paused();

            expect(paused).to.false;

            await lightNode.connect(wallet).togglePause(true);

            expect(await lightNode.paused()).to.true;

            await lightNode.connect(wallet).togglePause(false);

            expect(await lightNode.paused()).to.false;
        });

        it("updateBlockHeader() -> reverts paused ", async function () {
            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            await lightNode.connect(wallet).togglePause(true);

            await expect(
                lightNode.updateBlockHeader(
                    await lightNode.getHeadersBytes(data.addBlock1, data.quorumCert1, data.validators1)
                )
            ).to.be.revertedWith("Pausable: paused");
        });

        it("updateBlockHeader() -> correct ", async function () {
            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateBlockHeader(
                await lightNode.getHeadersBytes(data.addBlock1, data.quorumCert1, data.validators1)
            );

            let current = await lightNode.headerHeight();

            expect(current).to.eq(21729190);

            await lightNode.updateBlockHeader(
                await lightNode.getHeadersBytes(data.addBlock2, data.quorumCert2, data.validators2)
            );

            current = await lightNode.headerHeight();

            expect(current).to.eq(21729620);
        });

        it("verifyProofData() -> correct ", async function () {
            let [wallet] = await ethers.getSigners();

            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateBlockHeader(
                await lightNode.getHeadersBytes(data.addBlock1, data.quorumCert1, data.validators1)
            );

            await lightNode.updateBlockHeader(
                await lightNode.getHeadersBytes(data.addBlock2, data.quorumCert2, data.validators2)
            );

            let receiptProof = new ReceiptProof(data.txReceipt, data.proof.key, data.proof.proof);

            let proofData = new ProofData(data.proofHead, receiptProof, data.proof_q);

            let proofBytes = await lightNode.getBytes(proofData);

            let result = await lightNode.verifyProofData(proofBytes, { gasLimit: 20000000 });

            //  expect(result.success).to.true;
        });
    });
});
