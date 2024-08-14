import { time, loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
import { TxLog, ReceiptProof, TxReceipt, index2key, ProofData } from "../utils/Util";
import { keccak256 } from "ethers/lib/utils";

let chainId = 137;

describe("OracleV2", function () {
    // We define a fixture to reuse the same setup in every test.
    // We use loadFixture to run this setup once, snapshot that state,
    // and reset Hardhat Network to that snapshot in every test.
    async function deployFixture() {
        let [wallet] = await ethers.getSigners();

        const OracleV2 = await ethers.getContractFactory("OracleV2");

        const oracle = await OracleV2.deploy(wallet.address);

        await oracle.connect(wallet).deployed();

        return oracle;
    }

    describe("Deployment", function () {
        it("togglePause() -> reverts  only admin ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let oracle = await loadFixture(deployFixture);

            let paused = await oracle.paused();

            expect(paused).to.false;

            await expect(oracle.connect(other).togglePause()).to.be.revertedWith("Ownable: caller is not the owner");
        });

        it("togglePause() -> correct ", async function () {
            let [wallet, other] = await ethers.getSigners();

            let oracle = await loadFixture(deployFixture);

            let paused = await oracle.paused();

            expect(paused).to.false;

            await oracle.connect(wallet).togglePause();

            expect(await oracle.paused()).to.true;

            await oracle.connect(wallet).togglePause();

            expect(await oracle.paused()).to.false;
        });

        it("updateMultisg() -> correct ", async function () {
            let [wallet, addr1, addr2, addr3] = await ethers.getSigners();

            let oracle = await loadFixture(deployFixture);

            let signers = [addr1.address, addr2.address, addr3.address];

            let quorum = 4;

            let info = await oracle.multisigInfo();

            console.log(info);

            expect(info.quorum).eq(0);

            await expect(oracle.updateMultisg(quorum, signers)).to.be.reverted;

            quorum = 3;

            await oracle.updateMultisg(quorum, signers);

            info = await oracle.multisigInfo();

            console.log(info);

            expect(info.quorum).eq(3);

            quorum = 2;

            signers = [wallet.address, addr2.address, addr3.address];

            await oracle.updateMultisg(quorum, signers);

            info = await oracle.multisigInfo();

            console.log(info);

            expect(info.quorum).eq(2);
        });

        it("proposal", async function () {
            let [wallet, addr1, addr2, addr3] = await ethers.getSigners();

            let oracle = await loadFixture(deployFixture);

            let signers = [addr1.address, addr2.address, addr3.address];

            let quorum = 2;

            await oracle.updateMultisg(quorum, signers);

            let receiptRoot = "0x9d1a63e744550eebbb4d141e5b77c13cb1c21f40fb4f124bb9f161cea166b8ff";

            let blockNum = 12913052;

            let info = await oracle.multisigInfo();

            let hash = keccak256(
                ethers.utils.solidityPack(
                    ["bytes32", "bytes32", "uint256", "uint256"],
                    [receiptRoot, info.version, blockNum, chainId]
                )
            );

            let s1 = addr1.signMessage(ethers.utils.arrayify(hash));

            let s2 = addr2.signMessage(ethers.utils.arrayify(hash));

            let isProposaled = await oracle.isProposaled(chainId, info.version, blockNum, addr1.address);

            expect(isProposaled).to.be.eq(ethers.constants.HashZero);

            await expect(oracle.connect(addr2).proposal(chainId, blockNum, receiptRoot, s1)).to.be.reverted;

            await oracle.connect(addr1).proposal(chainId, blockNum, receiptRoot, s1);

            await expect(oracle.connect(addr1).proposal(chainId, blockNum, receiptRoot, s1)).to.be.reverted;

            isProposaled = await oracle.isProposaled(chainId, info.version, blockNum, addr1.address);

            expect(isProposaled).eq(receiptRoot);

            await expect(oracle.connect(addr2).recoverProposal(chainId, blockNum, addr1.address, 0)).to.be.reverted;

            await oracle.connect(addr1).recoverProposal(chainId, blockNum, addr1.address, 0);

            isProposaled = await oracle.isProposaled(chainId, info.version, blockNum, addr1.address);

            expect(isProposaled).eq(ethers.constants.HashZero);

            await oracle.connect(addr1).proposal(chainId, blockNum, receiptRoot, s1);

            isProposaled = await oracle.isProposaled(chainId, info.version, blockNum, addr1.address);

            expect(isProposaled).eq(receiptRoot);

            let p = await oracle.proposalInfo(chainId, blockNum, receiptRoot, info.version);

            expect(p.canVerify).to.be.false;

            await expect(oracle.connect(addr2).proposal(chainId, blockNum, receiptRoot, s2)).to.be.emit(oracle, "Meet");

            isProposaled = await oracle.isProposaled(chainId, info.version, blockNum, addr1.address);

            expect(isProposaled).eq(receiptRoot);

            p = await oracle.proposalInfo(chainId, blockNum, receiptRoot, info.version);

            expect(p.canVerify).to.be.true;

            console.log(p);
        });
    });
});
