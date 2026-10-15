import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
import { hexConcat, hexZeroPad, keccak256 } from "ethers/lib/utils";

const chainId = 137;
const LOG_ADDR = "0xb877E3562a660C7861117c2f1361A26ABaF19bEB";
const TOPIC0 = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
const TOPIC1 = "0x0000000000000000000000000000000000000000000000000000000000000001";
const TOPIC2 = "0x0000000000000000000000000000000000000000000000000000000000000002";

// addr(20) | reserved(4) | topic num(4) | data len(4) | topics | data
function encodeLog(addr: string, topics: string[], data: string): string {
    const dataLen = (data.length - 2) / 2;
    return hexConcat([
        addr,
        hexZeroPad("0x00", 4),
        hexZeroPad(BigNumber.from(topics.length).toHexString(), 4),
        hexZeroPad(BigNumber.from(dataLen).toHexString(), 4),
        ...topics,
        data,
    ]);
}

// position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
function position(blockNum = 12913052, txIndex = 0, logIndex = 0): BigNumber {
    return BigNumber.from(logIndex).shl(96).or(BigNumber.from(txIndex).shl(64)).or(blockNum);
}

describe("LightNodeV3 (more)", function () {
    async function deployFixture() {
        const [wallet, addr1, addr2, addr3, other] = await ethers.getSigners();

        const LightNode = await ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
        const impl = await LightNode.deploy();
        await impl.deployed();
        const Proxy = await ethers.getContractFactory("LightNodeProxy");
        const proxy = await Proxy.deploy(
            impl.address,
            LightNode.interface.encodeFunctionData("initialize", [chainId, wallet.address])
        );
        await proxy.deployed();
        const lightNode = LightNode.attach(proxy.address);

        return { lightNode, impl, LightNode, wallet, addr1, addr2, addr3, other };
    }

    // single fixture (nested / alternating fixtures invalidate hardhat snapshots), then configure the multisig
    async function deployConfigured() {
        const fx = await loadFixture(deployFixture);
        await fx.lightNode.updateMultisig(2, [fx.addr1.address, fx.addr2.address]);
        const info = await fx.lightNode.multisigInfo();
        return { ...fx, version: info.version as string };
    }

    async function makeProof(
        fx: Awaited<ReturnType<typeof deployConfigured>>,
        log: string,
        pos: BigNumber,
        opts: { signChainId?: number; signVersion?: string; proofType?: number } = {}
    ) {
        const receiptRoot = keccak256(log);
        const hash = keccak256(
            ethers.utils.solidityPack(
                ["bytes32", "bytes32", "uint256", "uint256"],
                [receiptRoot, opts.signVersion ?? fx.version, pos, opts.signChainId ?? chainId]
            )
        );
        const signatures = [
            await fx.addr1.signMessage(ethers.utils.arrayify(hash)),
            await fx.addr2.signMessage(ethers.utils.arrayify(hash)),
        ];
        return fx.lightNode.getBytes({
            proofType: opts.proofType ?? 1,
            position: pos,
            receiptRoot,
            signatures,
            encodeLog: log,
        });
    }

    describe("initialize", function () {
        it("sets chain id, node type and the roles of the admin", async function () {
            const { lightNode, wallet } = await loadFixture(deployFixture);
            expect(await lightNode.chainId()).to.equal(chainId);
            expect(await lightNode.nodeType()).to.equal(5);
            for (const role of ["MANAGER_ROLE", "UPGRADER_ROLE", "PAUSER_ROLE"]) {
                expect(await lightNode.hasRole(await lightNode[role](), wallet.address), role).to.be.true;
            }
            expect(await lightNode.hasRole(ethers.constants.HashZero, wallet.address)).to.be.true;
        });

        it("cannot be initialized twice", async function () {
            const { lightNode, wallet } = await loadFixture(deployFixture);
            await expect(lightNode.initialize(chainId, wallet.address)).to.be.revertedWith(
                "Initializable: contract is already initialized"
            );
        });

        it("the bare implementation is locked", async function () {
            const { impl, wallet } = await loadFixture(deployFixture);
            await expect(impl.initialize(chainId, wallet.address)).to.be.revertedWith(
                "Initializable: contract is already initialized"
            );
        });

        it("rejects a zero admin and a zero chain id", async function () {
            const { LightNode, wallet } = await loadFixture(deployFixture);
            const Proxy = await ethers.getContractFactory("LightNodeProxy");
            const impl = await LightNode.deploy();
            await expect(
                Proxy.deploy(
                    impl.address,
                    LightNode.interface.encodeFunctionData("initialize", [chainId, ethers.constants.AddressZero])
                )
            ).to.be.reverted;
            await expect(
                Proxy.deploy(impl.address, LightNode.interface.encodeFunctionData("initialize", [0, wallet.address]))
            ).to.be.revertedWith("invalid _chainId");
        });
    });

    describe("roles", function () {
        it("only the pauser can pause and only the manager can unpause", async function () {
            const { lightNode, wallet, other } = await loadFixture(deployFixture);
            await expect(lightNode.connect(other).pause()).to.be.reverted;
            await lightNode.pause();
            expect(await lightNode.paused()).to.be.true;

            await expect(lightNode.connect(other).unpause()).to.be.reverted;
            await lightNode.grantRole(await lightNode.MANAGER_ROLE(), other.address);
            await lightNode.connect(other).unpause();
            expect(await lightNode.paused()).to.be.false;
            // a manager that is not a pauser still cannot pause
            await expect(lightNode.connect(other).pause()).to.be.reverted;
            expect(wallet.address).to.not.equal(other.address);
        });

        it("only the manager can update the multisig", async function () {
            const { lightNode, addr1, other } = await loadFixture(deployFixture);
            await expect(lightNode.connect(other).updateMultisig(1, [addr1.address])).to.be.reverted;
        });

        it("isVerifiable follows the quorum and the pause state and ignores its arguments", async function () {
            const { lightNode, addr1, addr2 } = await loadFixture(deployFixture);
            expect(await lightNode.isVerifiable(1, ethers.constants.HashZero)).to.be.false;
            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);
            expect(await lightNode.isVerifiable(1, ethers.constants.HashZero)).to.be.true;
            expect(await lightNode.isVerifiable(999999, keccak256("0x01"))).to.be.true;
            await lightNode.pause();
            expect(await lightNode.isVerifiable(1, ethers.constants.HashZero)).to.be.false;
        });

        it("static views of the log based node", async function () {
            const { lightNode } = await loadFixture(deployFixture);
            expect(await lightNode.headerHeight()).to.equal(0);
            const range = await lightNode.verifiableHeaderRange();
            expect(range[0]).to.equal(0);
            expect(range[1]).to.equal(0);
            expect(await lightNode.clientState()).to.equal("0x");
            expect(await lightNode.finalizedState("0x1234")).to.equal("0x");
            await lightNode.updateBlockHeader("0x1234");
            await lightNode.updateLightClient("0x1234");
        });

        it("notifyLightClient emits ClientNotifySend", async function () {
            const { lightNode, other } = await loadFixture(deployFixture);
            const tx = await lightNode.connect(other).notifyLightClient(other.address, "0xabcd");
            const rc = await tx.wait();
            await expect(tx).to.emit(lightNode, "ClientNotifySend").withArgs(other.address, rc.blockNumber, "0xabcd");
        });
    });

    describe("updateMultisig", function () {
        it("sets quorum, signers and a version derived from them, and emits UpdateMultisig", async function () {
            const { lightNode, addr1, addr2, addr3 } = await loadFixture(deployFixture);
            const signers = [addr1.address, addr2.address, addr3.address];
            const version = keccak256(ethers.utils.solidityPack(["uint256", "address[]"], [2, signers]));

            await expect(lightNode.updateMultisig(2, signers))
                .to.emit(lightNode, "UpdateMultisig")
                .withArgs(version, 2, signers);

            const info = await lightNode.multisigInfo();
            expect(info.version).to.equal(version);
            expect(info.quorum).to.equal(2);
            expect(info.singers).to.deep.equal(signers);
        });

        it("replaces the previous signer set completely", async function () {
            const { lightNode, addr1, addr2, addr3 } = await loadFixture(deployFixture);
            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);
            await lightNode.updateMultisig(1, [addr3.address]);
            const info = await lightNode.multisigInfo();
            expect(info.singers).to.deep.equal([addr3.address]);
            expect(info.quorum).to.equal(1);
        });

        it("rejects quorum 0, quorum above the signer count, duplicates, zero address and empty sets", async function () {
            const { lightNode, addr1, addr2 } = await loadFixture(deployFixture);
            await expect(lightNode.updateMultisig(0, [addr1.address])).to.be.revertedWithCustomError(
                lightNode,
                "ECDSAMultisig_QuorumValueZero"
            );
            await expect(lightNode.updateMultisig(3, [addr1.address, addr2.address])).to.be.revertedWithCustomError(
                lightNode,
                "ECDSAMultisig_InsufficientSigners"
            );
            await expect(lightNode.updateMultisig(1, [addr1.address, addr1.address])).to.be.revertedWithCustomError(
                lightNode,
                "ECDSAMultisig_AddSignerFailed"
            );
            await expect(
                lightNode.updateMultisig(1, [ethers.constants.AddressZero])
            ).to.be.revertedWithCustomError(lightNode, "ECDSAMultisig_SignerZeroAddress");
            await expect(lightNode.updateMultisig(1, [])).to.be.revertedWithCustomError(
                lightNode,
                "ECDSAMultisig_InsufficientSigners"
            );
        });

        it("accepts 256 signers and rejects the 257th", async function () {
            const { lightNode } = await loadFixture(deployFixture);
            const signers = Array.from({ length: 256 }, (_, i) =>
                ethers.utils.getAddress(hexZeroPad(BigNumber.from(i + 1).toHexString(), 20))
            );
            await lightNode.updateMultisig(256, signers);
            expect((await lightNode.multisigInfo()).singers.length).to.equal(256);

            const tooMany = [...signers, ethers.utils.getAddress(hexZeroPad("0xffff", 20))];
            await expect(lightNode.updateMultisig(1, tooMany)).to.be.revertedWithCustomError(
                lightNode,
                "ECDSAMultisig_SignerLimitReached"
            );
        });

        it("a failed update leaves the previous configuration untouched", async function () {
            const { lightNode, addr1, addr2 } = await loadFixture(deployFixture);
            await lightNode.updateMultisig(2, [addr1.address, addr2.address]);
            const before = await lightNode.multisigInfo();
            await expect(lightNode.updateMultisig(1, [addr1.address, addr1.address])).to.be.reverted;
            const after = await lightNode.multisigInfo();
            expect(after.version).to.equal(before.version);
            expect(after.quorum).to.equal(before.quorum);
            expect(after.singers).to.deep.equal(before.singers);
        });

        it("[known quirk] the version depends on the signer order: reordering the same signers invalidates signatures", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            const proof = await makeProof(fx, log, position());
            expect((await fx.lightNode["verifyProofData(bytes)"](proof)).success).to.be.true;

            await fx.lightNode.updateMultisig(2, [fx.addr2.address, fx.addr1.address]); // same set, other order
            await expect(fx.lightNode["verifyProofData(bytes)"](proof)).to.be.reverted;
        });

        it("[known quirk] rotating back to the identical configuration makes old signatures valid again", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            const proof = await makeProof(fx, log, position());

            await fx.lightNode.updateMultisig(1, [fx.addr3.address]);
            await expect(fx.lightNode["verifyProofData(bytes)"](proof)).to.be.reverted;

            await fx.lightNode.updateMultisig(2, [fx.addr1.address, fx.addr2.address]);
            expect((await fx.lightNode["verifyProofData(bytes)"](proof)).success).to.be.true;
        });
    });

    describe("signature domain", function () {
        it("a signature for another source chain id is rejected", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            const proof = await makeProof(fx, log, position(), { signChainId: chainId + 1 });
            await expect(fx.lightNode["verifyProofData(bytes)"](proof)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("a signature for another multisig version is rejected", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            const proof = await makeProof(fx, log, position(), { signVersion: keccak256("0x01") });
            await expect(fx.lightNode["verifyProofData(bytes)"](proof)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("a signature for another position is rejected", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            const signed = await makeProof(fx, log, position(100, 0, 0));
            // same signatures, other block number
            const decoded = ethers.utils.defaultAbiCoder.decode(
                ["tuple(uint8 proofType,uint256 position,bytes32 receiptRoot,bytes[] signatures,bytes encodeLog)"],
                signed
            )[0];
            const forged = await fx.lightNode.getBytes({
                proofType: decoded.proofType,
                position: position(101, 0, 0),
                receiptRoot: decoded.receiptRoot,
                signatures: decoded.signatures,
                encodeLog: decoded.encodeLog,
            });
            await expect(fx.lightNode["verifyProofData(bytes)"](forged)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("the quorum must be reached", async function () {
            const fx = await loadFixture(deployFixture);
            await fx.lightNode.updateMultisig(2, [fx.addr1.address, fx.addr2.address]);
            const proof = await fx.lightNode.getBytes({
                proofType: 1,
                position: position(),
                receiptRoot: keccak256("0x1234"),
                signatures: [],
                encodeLog: "0x1234",
            });
            await expect(fx.lightNode["verifyProofData(bytes)"](proof)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_QuorumNotReached"
            );
        });

        it("verification fails while no multisig is configured", async function () {
            const fx = await loadFixture(deployFixture);
            const proof = await fx.lightNode.getBytes({
                proofType: 1,
                position: position(),
                receiptRoot: keccak256("0x1234"),
                signatures: [],
                encodeLog: "0x1234",
            });
            await expect(fx.lightNode["verifyProofData(bytes)"](proof)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_QuorumValueZero"
            );
        });
    });

    describe("log decoding", function () {
        const decode = (fx: any, log: string, pos = position()) =>
            makeProof(fx, log, pos).then((p) => fx.lightNode["verifyProofData(uint256,bytes)"](0, p));

        it("decodes a log with several topics and data", async function () {
            const fx = await deployConfigured();
            const data = "0x" + "ab".repeat(96);
            const r = await decode(fx, encodeLog(LOG_ADDR, [TOPIC0, TOPIC1, TOPIC2], data));
            expect(r.success).to.be.true;
            expect(r.log.addr).to.equal(LOG_ADDR);
            expect(r.log.topics).to.deep.equal([TOPIC0, TOPIC1, TOPIC2]);
            expect(r.log.data).to.equal(data);
        });

        it("decodes a log without topics", async function () {
            const fx = await deployConfigured();
            const data = "0x" + "cd".repeat(40);
            const r = await decode(fx, encodeLog(LOG_ADDR, [], data));
            expect(r.log.topics.length).to.equal(0);
            expect(r.log.data).to.equal(data);
        });

        it("decodes a log without data", async function () {
            const fx = await deployConfigured();
            const r = await decode(fx, encodeLog(LOG_ADDR, [TOPIC0], "0x"));
            expect(r.log.topics).to.deep.equal([TOPIC0]);
            expect(r.log.data).to.equal("0x");
        });

        it("decodes data whose length is not a multiple of 32", async function () {
            const fx = await deployConfigured();
            const data = "0x" + "ef".repeat(45);
            const r = await decode(fx, encodeLog(LOG_ADDR, [TOPIC0, TOPIC1], data));
            expect(r.log.data).to.equal(data);
        });

        it("the bytes overload returns the custom encoded log, not rlp logs", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            const proof = await makeProof(fx, log, position());
            const r = await fx.lightNode["verifyProofData(bytes)"](proof);
            expect(r.logs.toLowerCase()).to.equal(log.toLowerCase());
        });

        it("rejects logs that are shorter than the header, or whose length fields do not match", async function () {
            const fx = await deployConfigured();
            const bad = [
                "0x" + "11".repeat(31), // shorter than the 32 byte header
                encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32)) + "00", // one trailing byte
                encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32)).slice(0, -2), // one byte missing
                // topic number claims 0xffffffff topics
                hexConcat([LOG_ADDR, "0x00000000", "0xffffffff", "0x00000000"]),
                // data length claims 0xffffffff bytes
                hexConcat([LOG_ADDR, "0x00000000", "0x00000000", "0xffffffff"]),
            ];
            for (const log of bad) {
                await expect(decode(fx, log), log.slice(0, 20)).to.be.revertedWithCustomError(
                    fx.lightNode,
                    "LightNodeV3_Invalid_Log_Bytes"
                );
            }
        });

        it("[known issue] the log index passed to verifyProofData is ignored, the index inside the signed position is used", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x" + "11".repeat(32));
            // the signers attested log index 5 of the transaction
            const proof = await makeProof(fx, log, position(12913052, 3, 5));

            // callers asking for any other log index still receive that log, LightNodeV2 selects the log by index
            for (const requested of [0, 5, 7, 4294967295]) {
                const r = await fx.lightNode["verifyProofData(uint256,bytes)"](requested, proof);
                expect(r.success, `index ${requested}`).to.be.true;
                expect(r.log.addr).to.equal(LOG_ADDR);
            }
            const withCache = await fx.lightNode["verifyProofDataWithCache(bool,uint256,bytes)"](false, 0, proof);
            expect(withCache.success).to.be.true;
        });
    });

    describe("proof validation order and types", function () {
        it("an unsupported proof type reverts even with valid signatures, an unknown type fails to decode", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x");
            const mpt = await makeProof(fx, log, position(), { proofType: 0 });
            await expect(fx.lightNode["verifyProofData(bytes)"](mpt)).to.be.revertedWithCustomError(
                fx.lightNode,
                "LightNodeV3_Unsupport_Type"
            );

            const receiptRoot = keccak256(log);
            const encoded = ethers.utils.defaultAbiCoder.encode(
                ["tuple(uint8 proofType,uint256 position,bytes32 receiptRoot,bytes[] signatures,bytes encodeLog)"],
                [{ proofType: 2, position: position(), receiptRoot, signatures: [], encodeLog: log }]
            );
            await expect(fx.lightNode["verifyProofData(bytes)"](encoded)).to.be.reverted;
        });

        it("garbage proof bytes revert", async function () {
            const { lightNode } = await deployConfigured();
            await expect(lightNode["verifyProofData(bytes)"]("0x1234")).to.be.reverted;
            await expect(lightNode["verifyProofData(uint256,bytes)"](0, "0x")).to.be.reverted;
        });

        it("a log that does not hash to the signed root is reported as success=false on both overloads", async function () {
            const fx = await deployConfigured();
            const log = encodeLog(LOG_ADDR, [TOPIC0], "0x");
            const other = encodeLog(LOG_ADDR, [TOPIC1], "0x");
            const receiptRoot = keccak256(log);
            const hash = keccak256(
                ethers.utils.solidityPack(
                    ["bytes32", "bytes32", "uint256", "uint256"],
                    [receiptRoot, fx.version, position(), chainId]
                )
            );
            const signatures = [
                await fx.addr1.signMessage(ethers.utils.arrayify(hash)),
                await fx.addr2.signMessage(ethers.utils.arrayify(hash)),
            ];
            const proof = await fx.lightNode.getBytes({
                proofType: 1,
                position: position(),
                receiptRoot,
                signatures,
                encodeLog: other, // signatures are valid, the log is not the signed one
            });
            const a = await fx.lightNode["verifyProofData(bytes)"](proof);
            expect(a.success).to.be.false;
            expect(a.message).to.equal("invalid event bytes");
            const b = await fx.lightNode["verifyProofData(uint256,bytes)"](0, proof);
            expect(b.success).to.be.false;
            expect(b.log.addr).to.equal(ethers.constants.AddressZero);
        });
    });
});
