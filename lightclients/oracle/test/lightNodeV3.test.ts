import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { keccak256 } from "ethers/lib/utils";

let chainId = 137;

describe("LightNodeV3", function () {
    async function deployFixture() {
        let [wallet] = await ethers.getSigners();

        const LightNode = await ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
        const lightNode = await LightNode.deploy();
        await lightNode.deployed();

        const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");
        let initData = LightNode.interface.encodeFunctionData("initialize", [chainId, wallet.address]);
        const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);
        await lightNodeProxy.deployed();

        return LightNode.attach(lightNodeProxy.address);
    }

    async function makePosition(blockNum: number, txIndex: number, logIndex: number) {
        const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
        const oracleImpl = await Oracle.deploy();
        await oracleImpl.deployed();
        return oracleImpl.encodePosition(blockNum, txIndex, logIndex, 0);
    }

    describe("Deployment", function () {
        it("upgradeTo() -> reverts only upgrader", async function () {
            let [wallet, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            const LightNode = await ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            await expect(lightNode.connect(other).upgradeTo(newImplement.address)).to.be.revertedWith("only upgrade role");
        });

        it("upgradeTo() -> correct", async function () {
            let [wallet] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            const LightNode = await ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            let oldImplement = await lightNode.getImplementation();
            expect(oldImplement).to.not.eq(newImplement.address);

            await lightNode.connect(wallet).upgradeTo(newImplement.address);
            expect(await lightNode.getImplementation()).to.eq(newImplement.address);
        });

        it("grantRole() -> reverts only default admin", async function () {
            let [, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await expect(lightNode.connect(other).grantRole(await lightNode.MANAGER_ROLE(), other.address)).to.be.reverted;
        });

        it("grantRole() -> correct", async function () {
            let [wallet, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.connect(wallet).grantRole(await lightNode.MANAGER_ROLE(), other.address);
            expect(await lightNode.hasRole(await lightNode.MANAGER_ROLE(), other.address)).to.eq(true);
        });

        it("updateMultisig() -> correct", async function () {
            let [wallet, addr1, addr2, addr3] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            let signers = [addr1.address, addr2.address, addr3.address];
            let quorum = 4;

            let info = await lightNode.multisigInfo();
            expect(info.quorum).to.equal(0);

            await expect(lightNode.updateMultisig(quorum, signers)).to.be.reverted;

            quorum = 3;
            await lightNode.updateMultisig(quorum, signers);
            info = await lightNode.multisigInfo();
            expect(info.quorum).to.equal(3);

            quorum = 2;
            signers = [wallet.address, addr2.address, addr3.address];
            await lightNode.updateMultisig(quorum, signers);
            info = await lightNode.multisigInfo();
            expect(info.quorum).to.equal(2);
        });

        it("pause() -> blocks verification", async function () {
            let [wallet, addr1, addr2] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);

            const blockNum = 12913052;
            const txIndex = 0;
            const logIndex = 0;
            const position = await makePosition(blockNum, txIndex, logIndex);
            const encodeLog =
                "0xb877E3562a660C7861117c2f1361A26ABaF19bEB000000000000000300000020ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000feb2b97e4efce787c08086dc16ab69e0639113800000000000000000000000000000000000000000000000000057d124f8e7c713";
            const receiptRoot = ethers.utils.keccak256(encodeLog);

            let info = await lightNode.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = await addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = await addr2.signMessage(ethers.utils.arrayify(hash));

            let proofData = {
                proofType: 1,
                position,
                receiptRoot,
                signatures: [s1, s2],
                encodeLog,
            };

            let bytes = await lightNode.getBytes(proofData);

            await lightNode.connect(wallet).pause();

            await expect(lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWith("Pausable: paused");
            await expect(lightNode["verifyProofData(uint256,bytes)"](0, bytes)).to.be.revertedWith("Pausable: paused");
        });

        it("verifyProofData() log -> correct", async function () {
            let [, addr1, addr2] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);

            const blockNum = 12913052;
            const txIndex = 0;
            const logIndex = 0;
            const position = await makePosition(blockNum, txIndex, logIndex);
            const encodeLog =
                "0xb877E3562a660C7861117c2f1361A26ABaF19bEB000000000000000300000020ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000feb2b97e4efce787c08086dc16ab69e0639113800000000000000000000000000000000000000000000000000057d124f8e7c713";
            const receiptRoot = ethers.utils.keccak256(encodeLog);

            let info = await lightNode.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = await addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = await addr2.signMessage(ethers.utils.arrayify(hash));

            let proofData = {
                proofType: 1,
                position,
                receiptRoot,
                signatures: [s1, s2],
                encodeLog,
            };

            let bytes = await lightNode.getBytes(proofData);
            let result = await lightNode["verifyProofData(bytes)"](bytes);

            expect(result.success).to.be.true;
            expect(result.message).to.eq("");
            expect(result.logs.toLowerCase()).to.eq(encodeLog.toLowerCase());
        });

        it("verifyProofData() log index -> correct", async function () {
            let [, addr1, addr2] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);

            const blockNum = 12913052;
            const txIndex = 0;
            const logIndex = 0;
            const position = await makePosition(blockNum, txIndex, logIndex);
            const encodeLog =
                "0xb877E3562a660C7861117c2f1361A26ABaF19bEB000000000000000300000020ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000feb2b97e4efce787c08086dc16ab69e0639113800000000000000000000000000000000000000000000000000057d124f8e7c713";
            const receiptRoot = ethers.utils.keccak256(encodeLog);

            let info = await lightNode.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = await addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = await addr2.signMessage(ethers.utils.arrayify(hash));

            let proofData = {
                proofType: 1,
                position,
                receiptRoot,
                signatures: [s1, s2],
                encodeLog,
            };

            let bytes = await lightNode.getBytes(proofData);
            let result = await lightNode["verifyProofData(uint256,bytes)"](0, bytes);

            expect(result.success).to.be.true;
            expect(result.message).to.eq("");
            expect(result.log.addr).to.eq("0xb877E3562a660C7861117c2f1361A26ABaF19bEB");
            expect(result.log.topics.length).to.eq(3);
            expect(result.log.data).to.eq("0x0000000000000000000000000000000000000000000000000057d124f8e7c713");
        });

        it("verifyProofData() log index -> revert for malformed log bytes", async function () {
            let [, addr1, addr2] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);

            const blockNum = 12913052;
            const txIndex = 0;
            const logIndex = 0;
            const position = await makePosition(blockNum, txIndex, logIndex);
            const encodeLog = "0xb877E3562a660C7861117c2f1361A26ABaF19bEB000000000000000300000020";
            const receiptRoot = ethers.utils.keccak256(encodeLog);

            let info = await lightNode.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = await addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = await addr2.signMessage(ethers.utils.arrayify(hash));

            let proofData = {
                proofType: 1,
                position,
                receiptRoot,
                signatures: [s1, s2],
                encodeLog,
            };

            let bytes = await lightNode.getBytes(proofData);
            await expect(lightNode["verifyProofData(uint256,bytes)"](0, bytes)).to.be.revertedWithCustomError(
                lightNode,
                "LightNodeV3_Invalid_Log_Bytes"
            );
        });

        it("verifyProofData() -> revert for unsupported proof type", async function () {
            let [, addr1, addr2] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);

            const blockNum = 12913052;
            const txIndex = 0;
            const logIndex = 0;
            const position = await makePosition(blockNum, txIndex, logIndex);
            const encodeLog = "0x1234";
            const receiptRoot = ethers.utils.keccak256(encodeLog);

            let info = await lightNode.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = await addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = await addr2.signMessage(ethers.utils.arrayify(hash));

            let proofData = {
                proofType: 0,
                position,
                receiptRoot,
                signatures: [s1, s2],
                encodeLog,
            };

            let bytes = await lightNode.getBytes(proofData);
            await expect(lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                lightNode,
                "LightNodeV3_Unsupport_Type"
            );
        });

        it("verifyProofData() -> fail for invalid event bytes", async function () {
            let [, addr1, addr2] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);

            const blockNum = 12913052;
            const txIndex = 0;
            const logIndex = 0;
            const position = await makePosition(blockNum, txIndex, logIndex);
            const encodeLog = "0x1234";
            const receiptRoot = ethers.constants.HashZero;

            let info = await lightNode.multisigInfo();
            let hash = keccak256(
                ethers.utils.solidityPack(["bytes32", "bytes32", "uint256", "uint256"], [receiptRoot, info.version, position, chainId])
            );

            let s1 = await addr1.signMessage(ethers.utils.arrayify(hash));
            let s2 = await addr2.signMessage(ethers.utils.arrayify(hash));

            let proofData = {
                proofType: 1,
                position,
                receiptRoot,
                signatures: [s1, s2],
                encodeLog,
            };

            let bytes = await lightNode.getBytes(proofData);
            let result = await lightNode["verifyProofData(bytes)"](bytes);

            expect(result.success).to.be.false;
            expect(result.message).to.eq("invalid event bytes");
            expect(result.logs).to.eq("0x");
        });

        it("verifyProofData() -> revert for position greater than uint128", async function () {
            let lightNode = await loadFixture(deployFixture);

            let proofData = {
                proofType: 1,
                position: ethers.BigNumber.from(2).pow(128).toString(),
                receiptRoot: ethers.constants.HashZero,
                signatures: [],
                encodeLog: "0x",
            };

            let bytes = await lightNode.getBytes(proofData);
            await expect(lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                lightNode,
                "LightNodeV3_Invalid_Position"
            );
        });
    });
});
