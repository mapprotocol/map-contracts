import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { keccak256 } from "ethers/lib/utils";

let chainId = 137;

describe("OracleV3", function () {
    async function deployFixture() {
        let [wallet] = await ethers.getSigners();

        const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
        const oracle = await Oracle.deploy();
        await oracle.deployed();

        const OracleProxy = await ethers.getContractFactory("OracleProxy");
        let initData = Oracle.interface.encodeFunctionData("initialize", [wallet.address]);
        const oracleProxy = await OracleProxy.deploy(oracle.address, initData);
        await oracleProxy.deployed();

        return Oracle.attach(oracleProxy.address);
    }

    describe("Deployment", function () {
        it("pause() -> reverts only pauser", async function () {
            let [, other] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            expect(await oracle.paused()).to.false;
            await expect(oracle.connect(other).pause()).to.be.reverted;
        });

        it("pause()/unpause() -> correct", async function () {
            let [wallet] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            await oracle.connect(wallet).pause();
            expect(await oracle.paused()).to.true;

            await oracle.connect(wallet).unpause();
            expect(await oracle.paused()).to.false;
        });

        it("upgradeTo() -> reverts only upgrader", async function () {
            let [wallet, other] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
            const newImplement = await Oracle.connect(wallet).deploy();
            await newImplement.deployed();

            await expect(oracle.connect(other).upgradeTo(newImplement.address)).to.be.revertedWith("only upgrade role");
        });

        it("upgradeTo() -> correct", async function () {
            let [wallet] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
            const newImplement = await Oracle.connect(wallet).deploy();
            await newImplement.deployed();

            let oldImplement = await oracle.getImplementation();
            expect(oldImplement).to.not.eq(newImplement.address);

            await oracle.connect(wallet).upgradeTo(newImplement.address);
            expect(await oracle.getImplementation()).to.eq(newImplement.address);
        });

        it("updateMultisig() -> correct", async function () {
            let [wallet, addr1, addr2, addr3] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            let signers = [addr1.address, addr2.address, addr3.address];
            let quorum = 4;

            let info = await oracle.multisigInfo();
            expect(info.quorum).to.equal(0);

            await expect(oracle.updateMultisig(quorum, signers)).to.be.reverted;

            quorum = 3;
            await oracle.updateMultisig(quorum, signers);
            info = await oracle.multisigInfo();
            expect(info.quorum).to.equal(3);

            quorum = 2;
            signers = [wallet.address, addr2.address, addr3.address];
            await oracle.updateMultisig(quorum, signers);
            info = await oracle.multisigInfo();
            expect(info.quorum).to.equal(2);
        });

        it("proposal", async function () {
            let [, addr1, addr2, addr3] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            let signers = [addr1.address, addr2.address, addr3.address];
            await oracle.updateMultisig(2, signers);

            let receiptRoot = "0x9d1a63e744550eebbb4d141e5b77c13cb1c21f40fb4f124bb9f161cea166b8ff";
            let blockNum = 12913052;
            let txIndex = 0;
            let logIndex = 0;
            let position = await oracle.encodePosition(blockNum, txIndex, logIndex, 0);

            let info = await oracle.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = addr2.signMessage(ethers.utils.arrayify(hash));

            let isProposed = await oracle["isProposed(uint256,bytes32,uint256,address)"](
                chainId,
                info.version,
                position,
                addr1.address
            );
            expect(isProposed).to.equal(ethers.constants.HashZero);

            await expect(
                oracle.connect(addr2)["propose(uint256,uint256,bytes32,bytes)"](chainId, position, receiptRoot, s1)
            ).to.be.reverted;

            await oracle.connect(addr1)["propose(uint256,uint256,bytes32,bytes)"](chainId, position, receiptRoot, s1);

            isProposed = await oracle["isProposed(uint256,bytes32,uint256,address)"](
                chainId,
                info.version,
                position,
                addr1.address
            );
            expect(isProposed).to.equal(receiptRoot);

            await expect(
                oracle.connect(addr2)["recoverProposal(uint256,uint256,address,uint256)"](chainId, position, addr1.address, 0)
            ).to.be.reverted;

            await oracle.connect(addr1)["recoverProposal(uint256,uint256,address,uint256)"](chainId, position, addr1.address, 0);

            isProposed = await oracle["isProposed(uint256,bytes32,uint256,address)"](
                chainId,
                info.version,
                position,
                addr1.address
            );
            expect(isProposed).to.equal(ethers.constants.HashZero);

            await oracle.connect(addr1)["propose(uint256,uint256,bytes32,bytes)"](chainId, position, receiptRoot, s1);

            let p = await oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](chainId, position, receiptRoot, info.version);
            expect(p.canVerify).to.be.false;

            await expect(
                oracle.connect(addr2)["propose(uint256,uint256,bytes32,bytes)"](chainId, position, receiptRoot, s2)
            ).to.be.emit(oracle, "Meet");

            p = await oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](chainId, position, receiptRoot, info.version);
            expect(p.canVerify).to.be.true;
        });

        it("proposal overloads -> txIndex participates in position", async function () {
            let [, addr1, addr2, addr3] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            await oracle.updateMultisig(2, [addr1.address, addr2.address, addr3.address]);

            let receiptRoot = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let blockNum = 12913052;
            let txIndex = 7;
            let logIndex = 3;

            let position = await oracle.encodePosition(blockNum, txIndex, logIndex, 0);
            let decoded = await oracle.decodePosition(position);
            expect(decoded.blockNum).to.equal(blockNum);
            expect(decoded.txIndex).to.equal(txIndex);
            expect(decoded.logIndex).to.equal(logIndex);

            let info = await oracle.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = addr2.signMessage(ethers.utils.arrayify(hash));

            await oracle.connect(addr1)["propose(uint256,uint256,uint256,uint256,uint256,bytes32,bytes)"](
                chainId,
                blockNum,
                txIndex,
                logIndex,
                0,
                receiptRoot,
                s1
            );
            await oracle.connect(addr2)["propose(uint256,uint256,bytes32,bytes)"](chainId, position, receiptRoot, s2);

            expect(
                await oracle["isProposed(uint256,bytes32,uint256,uint256,uint256,uint256,address)"](
                    chainId,
                    info.version,
                    blockNum,
                    txIndex,
                    logIndex,
                    0,
                    addr1.address
                )
            ).to.equal(receiptRoot);
        });

        it("proposal -> reject position greater than uint128", async function () {
            let [, addr1, addr2, addr3] = await ethers.getSigners();
            let oracle = await loadFixture(deployFixture);

            await oracle.updateMultisig(2, [addr1.address, addr2.address, addr3.address]);

            let receiptRoot = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
            let invalidPosition = ethers.BigNumber.from(2).pow(128);

            await expect(
                oracle.connect(addr1)["propose(uint256,uint256,bytes32,bytes)"](chainId, invalidPosition, receiptRoot, "0x")
            ).to.be.revertedWithCustomError(oracle, "invalid_proposal_param");
        });
    });
});
