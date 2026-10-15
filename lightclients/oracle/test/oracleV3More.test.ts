import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
import { keccak256 } from "ethers/lib/utils";

const chainId = 137;
const ROOT = "0x9d1a63e744550eebbb4d141e5b77c13cb1c21f40fb4f124bb9f161cea166b8ff";
const ROOT2 = "0x1111111111111111111111111111111111111111111111111111111111111111";

describe("OracleV3 (more)", function () {
    async function deployFixture() {
        const [wallet, addr1, addr2, addr3, other] = await ethers.getSigners();

        const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
        const impl = await Oracle.deploy();
        await impl.deployed();
        const OracleProxy = await ethers.getContractFactory("OracleProxy");
        const proxy = await OracleProxy.deploy(
            impl.address,
            Oracle.interface.encodeFunctionData("initialize", [wallet.address])
        );
        await proxy.deployed();
        const oracle = Oracle.attach(proxy.address);
        const position: BigNumber = await oracle.encodePosition(12913052, 7, 3, 0);

        return { oracle, impl, wallet, addr1, addr2, addr3, other, position };
    }

    // single fixture (alternating fixtures invalidate hardhat snapshots), then configure the multisig
    async function configured(quorum = 2) {
        const fx = await loadFixture(deployFixture);
        await fx.oracle.updateMultisig(quorum, [fx.addr1.address, fx.addr2.address, fx.addr3.address]);
        const info = await fx.oracle.multisigInfo();
        const version = info.version as string;

        const sign = async (signer: any, position: BigNumber, root = ROOT, cid = chainId, ver = version) =>
            signer.signMessage(
                ethers.utils.arrayify(
                    keccak256(
                        ethers.utils.solidityPack(
                            ["bytes32", "bytes32", "uint256", "uint256"],
                            [root, ver, position, cid]
                        )
                    )
                )
            );
        const propose = async (signer: any, position: BigNumber, root = ROOT, cid = chainId) =>
            fx.oracle
                .connect(signer)
                ["propose(uint256,uint256,bytes32,bytes)"](cid, position, root, await sign(signer, position, root, cid));
        const isProposed = (position: BigNumber, signer: string) =>
            fx.oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, position, signer);
        const info2 = (position: BigNumber, root = ROOT, ver = ethers.constants.HashZero) =>
            fx.oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](chainId, position, root, ver);
        const recover = (caller: any, position: BigNumber, signer: string, index: number) =>
            fx.oracle
                .connect(caller)
                ["recoverProposal(uint256,uint256,address,uint256)"](chainId, position, signer, index);

        return { ...fx, version, sign, propose, isProposed, info2, recover };
    }

    describe("propose", function () {
        it("only the signer itself can submit its signature", async function () {
            const fx = await configured();
            const s1 = await fx.sign(fx.addr1, fx.position);
            await expect(
                fx.oracle
                    .connect(fx.addr2)
                    ["propose(uint256,uint256,bytes32,bytes)"](chainId, fx.position, ROOT, s1)
            ).to.be.revertedWithCustomError(fx.oracle, "only_signer");
        });

        it("rejects a signature from an account outside of the multisig", async function () {
            const fx = await configured();
            await expect(fx.propose(fx.other, fx.position)).to.be.revertedWithCustomError(
                fx.oracle,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("rejects a zero chain id and a zero root", async function () {
            const fx = await configured();
            await expect(fx.propose(fx.addr1, fx.position, ROOT, 0)).to.be.revertedWithCustomError(
                fx.oracle,
                "invalid_proposal_param"
            );
            await expect(fx.propose(fx.addr1, fx.position, ethers.constants.HashZero)).to.be.revertedWithCustomError(
                fx.oracle,
                "invalid_proposal_param"
            );
        });

        it("a signature made for another chain id or version is rejected", async function () {
            const fx = await configured();
            const wrongChain = await fx.sign(fx.addr1, fx.position, ROOT, chainId + 1);
            await expect(
                fx.oracle
                    .connect(fx.addr1)
                    ["propose(uint256,uint256,bytes32,bytes)"](chainId, fx.position, ROOT, wrongChain)
            ).to.be.revertedWithCustomError(fx.oracle, "ECDSAMultisig_RecoveredSignerNotAuthorized");

            const wrongVersion = await fx.sign(fx.addr1, fx.position, ROOT, chainId, keccak256("0x01"));
            await expect(
                fx.oracle
                    .connect(fx.addr1)
                    ["propose(uint256,uint256,bytes32,bytes)"](chainId, fx.position, ROOT, wrongVersion)
            ).to.be.revertedWithCustomError(fx.oracle, "ECDSAMultisig_RecoveredSignerNotAuthorized");
        });

        it("a signer can only propose once per position, even for a different root", async function () {
            const fx = await configured(3);
            await fx.propose(fx.addr1, fx.position, ROOT);
            await expect(fx.propose(fx.addr1, fx.position, ROOT)).to.be.revertedWithCustomError(
                fx.oracle,
                "already_proposal"
            );
            await expect(fx.propose(fx.addr1, fx.position, ROOT2)).to.be.revertedWithCustomError(
                fx.oracle,
                "already_proposal"
            );
        });

        it("emits Meet with the collected signatures when the quorum is reached", async function () {
            const fx = await configured(2);
            const s1 = await fx.sign(fx.addr1, fx.position);
            const s2 = await fx.sign(fx.addr2, fx.position);
            await fx.propose(fx.addr1, fx.position);
            await expect(fx.propose(fx.addr2, fx.position))
                .to.emit(fx.oracle, "Meet")
                .withArgs(chainId, fx.position, ROOT, [s1, s2]);
        });

        it("no more proposals for a root once the quorum is met, other roots are still accepted", async function () {
            const fx = await configured(2);
            await fx.propose(fx.addr1, fx.position, ROOT);
            await fx.propose(fx.addr2, fx.position, ROOT);
            await expect(fx.propose(fx.addr3, fx.position, ROOT)).to.be.revertedWithCustomError(
                fx.oracle,
                "already_meet"
            );
            // a conflicting root from the third signer is stored separately and does not meet
            await expect(fx.propose(fx.addr3, fx.position, ROOT2)).to.emit(fx.oracle, "Proposal");
            expect((await fx.info2(fx.position, ROOT2)).canVerify).to.be.false;
            expect((await fx.info2(fx.position, ROOT)).canVerify).to.be.true;
        });

        it("a quorum of one meets with the first proposal", async function () {
            const fx = await configured(1);
            await expect(fx.propose(fx.addr1, fx.position)).to.emit(fx.oracle, "Meet");
        });

        it("the same signer may propose different positions and chains independently", async function () {
            const fx = await configured(2);
            const p2: BigNumber = await fx.oracle.encodePosition(12913053, 0, 0, 0);
            await fx.propose(fx.addr1, fx.position);
            await fx.propose(fx.addr1, p2);
            await fx.propose(fx.addr1, fx.position, ROOT, chainId + 1);
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ROOT);
            expect(await fx.isProposed(p2, fx.addr1.address)).to.equal(ROOT);
        });

        it("is blocked while paused", async function () {
            const fx = await configured();
            await fx.oracle.pause();
            await expect(fx.propose(fx.addr1, fx.position)).to.be.revertedWith("Pausable: paused");
            await fx.oracle.unpause();
            await fx.propose(fx.addr1, fx.position);
        });
    });

    describe("batchPropose", function () {
        it("stores every item of the batch", async function () {
            const fx = await configured(2);
            const p2: BigNumber = await fx.oracle.encodePosition(12913053, 1, 1, 0);
            await fx.oracle.connect(fx.addr1).batchPropose([
                { chainId, position: fx.position, rootHash: ROOT, signature: await fx.sign(fx.addr1, fx.position) },
                { chainId, position: p2, rootHash: ROOT2, signature: await fx.sign(fx.addr1, p2, ROOT2) },
            ]);
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ROOT);
            expect(await fx.isProposed(p2, fx.addr1.address)).to.equal(ROOT2);
        });

        it("reverts as a whole when one item is invalid", async function () {
            const fx = await configured(2);
            const p2: BigNumber = await fx.oracle.encodePosition(12913053, 1, 1, 0);
            await expect(
                fx.oracle.connect(fx.addr1).batchPropose([
                    { chainId, position: fx.position, rootHash: ROOT, signature: await fx.sign(fx.addr1, fx.position) },
                    // signature of another signer
                    { chainId, position: p2, rootHash: ROOT, signature: await fx.sign(fx.addr2, p2) },
                ])
            ).to.be.revertedWithCustomError(fx.oracle, "only_signer");
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ethers.constants.HashZero);
        });

        it("an empty batch is a no-op and the batch is blocked while paused", async function () {
            const fx = await configured();
            await fx.oracle.connect(fx.addr1).batchPropose([]);
            await fx.oracle.pause();
            await expect(fx.oracle.connect(fx.addr1).batchPropose([])).to.be.revertedWith("Pausable: paused");
        });
    });

    describe("recoverProposal", function () {
        it("only the signer or a manager can recover", async function () {
            const fx = await configured(3);
            await fx.propose(fx.addr1, fx.position);
            await expect(fx.recover(fx.other, fx.position, fx.addr1.address, 0)).to.be.revertedWithCustomError(
                fx.oracle,
                "only_signer_or_owner"
            );
            await expect(fx.recover(fx.addr2, fx.position, fx.addr1.address, 0)).to.be.revertedWithCustomError(
                fx.oracle,
                "only_signer_or_owner"
            );
            // the manager (deployer) may recover on behalf of the signer
            await fx.recover(fx.wallet, fx.position, fx.addr1.address, 0);
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ethers.constants.HashZero);
        });

        it("emits RecoverProposal and removes the signature", async function () {
            const fx = await configured(3);
            await fx.propose(fx.addr1, fx.position);
            await fx.propose(fx.addr2, fx.position);
            await expect(fx.recover(fx.addr1, fx.position, fx.addr1.address, 0))
                .to.emit(fx.oracle, "RecoverProposal")
                .withArgs(chainId, fx.position, fx.addr1.address, 0);

            const info = await fx.info2(fx.position);
            expect(info.singers).to.deep.equal([fx.addr2.address]); // swap and pop moved the last entry
            expect(info.signatures.length).to.equal(1);
        });

        it("reverts when nothing was proposed, for a wrong index and for an out-of-range index", async function () {
            const fx = await configured(3);
            await expect(fx.recover(fx.addr1, fx.position, fx.addr1.address, 0)).to.be.revertedWithCustomError(
                fx.oracle,
                "not_proposal"
            );

            await fx.propose(fx.addr1, fx.position);
            await fx.propose(fx.addr2, fx.position);
            await expect(fx.recover(fx.addr1, fx.position, fx.addr1.address, 1)).to.be.revertedWithCustomError(
                fx.oracle,
                "singer_mismatching"
            );
            await expect(fx.recover(fx.addr1, fx.position, fx.addr1.address, 5)).to.be.revertedWithCustomError(
                fx.oracle,
                "signatures_out_bond"
            );
        });

        it("after a recovery the signer can propose again, also a different root", async function () {
            const fx = await configured(3);
            await fx.propose(fx.addr1, fx.position, ROOT);
            await fx.recover(fx.addr1, fx.position, fx.addr1.address, 0);
            await fx.propose(fx.addr1, fx.position, ROOT2);
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ROOT2);
        });

        it("recovering after the quorum was met re-opens the root for proposals", async function () {
            const fx = await configured(2);
            await fx.propose(fx.addr1, fx.position);
            await fx.propose(fx.addr2, fx.position);
            expect((await fx.info2(fx.position)).canVerify).to.be.true;

            await fx.recover(fx.addr2, fx.position, fx.addr2.address, 1);
            expect((await fx.info2(fx.position)).canVerify).to.be.false;
            await expect(fx.propose(fx.addr3, fx.position)).to.emit(fx.oracle, "Meet");
        });

        it("is allowed while paused", async function () {
            const fx = await configured(3);
            await fx.propose(fx.addr1, fx.position);
            await fx.oracle.pause();
            await fx.recover(fx.addr1, fx.position, fx.addr1.address, 0);
        });

        it("the block number overload of recoverProposal works", async function () {
            const fx = await configured(3);
            await fx.propose(fx.addr1, fx.position);
            await fx.oracle
                .connect(fx.addr1)
                ["recoverProposal(uint256,uint256,uint256,uint256,uint256,address,uint256)"](
                    chainId,
                    12913052,
                    7,
                    3,
                    0,
                    fx.addr1.address,
                    0
                );
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ethers.constants.HashZero);
        });
    });

    describe("multisig updates", function () {
        it("only the manager can update the multisig", async function () {
            const fx = await configured();
            await expect(fx.oracle.connect(fx.other).updateMultisig(1, [fx.addr1.address])).to.be.reverted;
        });

        it("proposals of the old version are kept but can no longer be verified", async function () {
            const fx = await configured(1);
            await fx.propose(fx.addr1, fx.position);
            expect((await fx.info2(fx.position)).canVerify).to.be.true;

            await fx.oracle.updateMultisig(1, [fx.addr1.address, fx.addr2.address]);
            // current version: nothing collected yet
            expect((await fx.info2(fx.position)).singers.length).to.equal(0);
            // old version: signatures are still readable, but not verifiable
            const old = await fx.info2(fx.position, ROOT, fx.version);
            expect(old.singers).to.deep.equal([fx.addr1.address]);
            expect(old.canVerify).to.be.false;
        });

        it("a signer can propose again under the new version", async function () {
            const fx = await configured(1);
            await fx.propose(fx.addr1, fx.position);
            await fx.oracle.updateMultisig(1, [fx.addr1.address, fx.addr2.address]);

            const info = await fx.oracle.multisigInfo();
            const s = await fx.sign(fx.addr1, fx.position, ROOT, chainId, info.version);
            await expect(
                fx.oracle
                    .connect(fx.addr1)
                    ["propose(uint256,uint256,bytes32,bytes)"](chainId, fx.position, ROOT, s)
            ).to.emit(fx.oracle, "Meet");
        });

        it("a removed signer can no longer propose", async function () {
            const fx = await configured(1);
            await fx.oracle.updateMultisig(1, [fx.addr2.address]);
            const info = await fx.oracle.multisigInfo();
            const s = await fx.sign(fx.addr1, fx.position, ROOT, chainId, info.version);
            await expect(
                fx.oracle
                    .connect(fx.addr1)
                    ["propose(uint256,uint256,bytes32,bytes)"](chainId, fx.position, ROOT, s)
            ).to.be.revertedWithCustomError(fx.oracle, "ECDSAMultisig_RecoveredSignerNotAuthorized");
        });

        it("[known issue] proposalInfo reports canVerify = true when no multisig is configured (quorum 0)", async function () {
            const fx = await loadFixture(deployFixture);
            const info = await fx.oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](
                chainId,
                fx.position,
                ROOT,
                ethers.constants.HashZero
            );
            expect(info.singers.length).to.equal(0);
            expect(info.canVerify).to.be.true;
        });
    });

    describe("position helpers", function () {
        it("encodePosition / decodePosition round trip, including the maximum values", async function () {
            const { oracle } = await loadFixture(deployFixture);
            const max64 = BigNumber.from(2).pow(64).sub(1);
            const max32 = 2 ** 32 - 1;
            for (const [b, t, l] of [
                [1, 0, 0],
                [12913052, 7, 3],
                [max64, max32, max32],
            ] as [number | BigNumber, number, number][]) {
                const p: BigNumber = await oracle.encodePosition(b, t, l, 0);
                const d = await oracle.decodePosition(p);
                expect(d.blockNum).to.equal(b);
                expect(d.txIndex).to.equal(t);
                expect(d.logIndex).to.equal(l);
                expect(d.expiry).to.equal(0);
            }
        });

        it("encodePosition layout: log index << 96 | tx index << 64 | block number", async function () {
            const { oracle } = await loadFixture(deployFixture);
            const p: BigNumber = await oracle.encodePosition(5, 6, 7, 0);
            expect(p).to.equal(BigNumber.from(7).shl(96).or(BigNumber.from(6).shl(64)).or(5));
        });

        it("encodePosition rejects block 0 and out-of-range fields", async function () {
            const { oracle } = await loadFixture(deployFixture);
            await expect(oracle.encodePosition(0, 0, 0, 0)).to.be.revertedWithCustomError(oracle, "invalid_proposal_param");
            await expect(oracle.encodePosition(BigNumber.from(2).pow(64), 0, 0, 0)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
            await expect(oracle.encodePosition(1, 2 ** 32, 0, 0)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
            await expect(oracle.encodePosition(1, 0, 2 ** 32, 0)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
        });

        it("the block number overloads and the position overloads address the same record", async function () {
            const fx = await configured(2);
            await fx.propose(fx.addr1, fx.position);
            const byBlock = await fx.oracle["isProposed(uint256,bytes32,uint256,uint256,uint256,uint256,address)"](
                chainId,
                fx.version,
                12913052,
                7,
                3,
                0,
                fx.addr1.address
            );
            expect(byBlock).to.equal(ROOT);

            const byBlockInfo = await fx.oracle["proposalInfo(uint256,uint256,uint256,uint256,uint256,bytes32,bytes32)"](
                chainId,
                12913052,
                7,
                3,
                0,
                ROOT,
                ethers.constants.HashZero
            );
            expect(byBlockInfo.singers).to.deep.equal([fx.addr1.address]);
        });

        it("isProposed is empty for another chain id", async function () {
            const fx = await configured(2);
            await fx.propose(fx.addr1, fx.position);
            expect(
                await fx.oracle["isProposed(uint256,bytes32,uint256,address)"](
                    chainId + 1,
                    fx.version,
                    fx.position,
                    fx.addr1.address
                )
            ).to.equal(ethers.constants.HashZero);
        });
    });

    describe("initialize and upgrade", function () {
        it("cannot be initialized twice, the implementation is locked and a zero admin is rejected", async function () {
            const fx = await loadFixture(deployFixture);
            await expect(fx.oracle.initialize(fx.wallet.address)).to.be.revertedWith(
                "Initializable: contract is already initialized"
            );
            await expect(fx.impl.initialize(fx.wallet.address)).to.be.revertedWith(
                "Initializable: contract is already initialized"
            );

            const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
            const OracleProxy = await ethers.getContractFactory("OracleProxy");
            await expect(
                OracleProxy.deploy(
                    fx.impl.address,
                    Oracle.interface.encodeFunctionData("initialize", [ethers.constants.AddressZero])
                )
            ).to.be.reverted;
        });

        it("storage survives an upgrade", async function () {
            const fx = await configured(2);
            await fx.propose(fx.addr1, fx.position);

            const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
            const newImpl = await Oracle.deploy();
            await fx.oracle.upgradeTo(newImpl.address);

            expect(await fx.oracle.getImplementation()).to.equal(newImpl.address);
            expect(await fx.isProposed(fx.position, fx.addr1.address)).to.equal(ROOT);
            expect((await fx.oracle.multisigInfo()).quorum).to.equal(2);
        });
    });
});
