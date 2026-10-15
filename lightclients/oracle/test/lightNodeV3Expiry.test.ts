import { loadFixture, time } from "@nomicfoundation/hardhat-network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
import { keccak256 } from "ethers/lib/utils";

const chainId = 137;
const DAY = 24 * 60 * 60;

// position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
const ENCODE_LOG =
    "0xb877E3562a660C7861117c2f1361A26ABaF19bEB000000000000000300000020ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000feb2b97e4efce787c08086dc16ab69e0639113800000000000000000000000000000000000000000000000000057d124f8e7c713";

function lowPosition(blockNum: number | BigNumber = 12913052, txIndex = 0, logIndex = 0): BigNumber {
    return BigNumber.from(logIndex).shl(96).or(BigNumber.from(txIndex).shl(64)).or(BigNumber.from(blockNum));
}

function withExpiry(expiry: number | BigNumber, low: BigNumber = lowPosition()): BigNumber {
    return BigNumber.from(expiry).shl(128).or(low);
}

describe("LightNodeV3 signature expiry", function () {
    async function deployFixture() {
        const [wallet, addr1, addr2] = await ethers.getSigners();

        const LightNode = await ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
        const impl = await LightNode.deploy();
        await impl.deployed();

        const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");
        const initData = LightNode.interface.encodeFunctionData("initialize", [chainId, wallet.address]);
        const proxy = await LightNodeProxy.deploy(impl.address, initData);
        await proxy.deployed();

        const lightNode = LightNode.attach(proxy.address);
        await lightNode.updateMultisig(2, [addr1.address, addr2.address]);
        const info = await lightNode.multisigInfo();

        return { lightNode, version: info.version as string, wallet, addr1, addr2 };
    }

    // `signedPosition` is what the signers signed, `position` is what is submitted (they differ in tamper tests)
    async function makeProof(
        fx: Awaited<ReturnType<typeof deployFixture>>,
        position: BigNumber,
        opts: { signedPosition?: BigNumber; encodeLog?: string; receiptRoot?: string; signatures?: string[] } = {}
    ) {
        const encodeLog = opts.encodeLog ?? ENCODE_LOG;
        const receiptRoot = opts.receiptRoot ?? keccak256(encodeLog);
        const signedPosition = opts.signedPosition ?? position;
        const hash = keccak256(
            ethers.utils.solidityPack(
                ["bytes32", "bytes32", "uint256", "uint256"],
                [receiptRoot, fx.version, signedPosition, chainId]
            )
        );
        const signatures = opts.signatures ?? [
            await fx.addr1.signMessage(ethers.utils.arrayify(hash)),
            await fx.addr2.signMessage(ethers.utils.arrayify(hash)),
        ];
        return fx.lightNode.getBytes({ proofType: 1, position, receiptRoot, signatures, encodeLog });
    }

    // every verification entry point of the light node
    const entryPoints: { name: string; call: (ln: any, bytes: string) => Promise<any> }[] = [
        { name: "verifyProofData(bytes)", call: (ln, b) => ln["verifyProofData(bytes)"](b) },
        { name: "verifyProofData(uint256,bytes)", call: (ln, b) => ln["verifyProofData(uint256,bytes)"](0, b) },
        { name: "verifyProofDataWithCache(bytes)", call: (ln, b) => ln["verifyProofDataWithCache(bytes)"](b) },
        {
            name: "verifyProofDataWithCache(bool,uint256,bytes)",
            call: (ln, b) => ln["verifyProofDataWithCache(bool,uint256,bytes)"](false, 0, b),
        },
    ];

    describe("expiry == 0 (never expires)", function () {
        it("is valid and stays valid forever", async function () {
            const fx = await loadFixture(deployFixture);
            const bytes = await makeProof(fx, lowPosition());

            const result = await fx.lightNode["verifyProofData(bytes)"](bytes);
            expect(result.success).to.be.true;

            await time.increase(100 * 365 * DAY);

            for (const ep of entryPoints) {
                const r = await ep.call(fx.lightNode, bytes);
                expect(r.success, ep.name).to.be.true;
            }
        });

        it("behaves exactly like a position without the expiry field", async function () {
            const fx = await loadFixture(deployFixture);
            const position = withExpiry(0);
            expect(position).to.equal(lowPosition());

            const bytes = await makeProof(fx, position);
            const result = await fx.lightNode["verifyProofData(bytes)"](bytes);
            expect(result.success).to.be.true;
            expect(result.logs.toLowerCase()).to.eq(ENCODE_LOG.toLowerCase());
        });
    });

    describe("expiry in the future", function () {
        it("verifies on every entry point and returns the log", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const bytes = await makeProof(fx, withExpiry(expiry));

            for (const ep of entryPoints) {
                const r = await ep.call(fx.lightNode, bytes);
                expect(r.success, ep.name).to.be.true;
                expect(r.message, ep.name).to.eq("");
            }

            const byLogs = await fx.lightNode["verifyProofData(bytes)"](bytes);
            expect(byLogs.logs.toLowerCase()).to.eq(ENCODE_LOG.toLowerCase());

            const byLog = await fx.lightNode["verifyProofData(uint256,bytes)"](0, bytes);
            expect(byLog.log.addr).to.eq("0xb877E3562a660C7861117c2f1361A26ABaF19bEB");
            expect(byLog.log.topics.length).to.eq(3);
        });

        it("a long lived proof is valid until the expiry and expired right after", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 30 * DAY;
            const bytes = await makeProof(fx, withExpiry(expiry));

            await time.increaseTo(expiry - DAY);
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;

            await time.increaseTo(expiry + 1);
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                fx.lightNode,
                "LightNodeV3_Signature_Expired"
            );
        });

        it("the maximum expiry (2^128 - 1) never expires in practice", async function () {
            const fx = await loadFixture(deployFixture);
            const bytes = await makeProof(fx, withExpiry(BigNumber.from(2).pow(128).sub(1)));

            await time.increase(1000 * 365 * DAY);
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;
        });

        it("expiry bits do not leak into / from block number, tx index and log index", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            // all low fields at their maximum
            const low = lowPosition(BigNumber.from(2).pow(64).sub(1), 2 ** 32 - 1, 2 ** 32 - 1);
            const position = withExpiry(expiry, low);
            expect(position.shr(128)).to.equal(expiry);
            expect(position.mask(128)).to.equal(low);

            const bytes = await makeProof(fx, position);
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;
        });

        it("an invalid event bytes result is still reported as success=false", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const bytes = await makeProof(fx, withExpiry(expiry), {
                receiptRoot: ethers.constants.HashZero,
                encodeLog: "0x1234",
            });

            const result = await fx.lightNode["verifyProofData(bytes)"](bytes);
            expect(result.success).to.be.false;
            expect(result.message).to.eq("invalid event bytes");
        });
    });

    describe("expired", function () {
        it("reverts with LightNodeV3_Signature_Expired on every entry point", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            const bytes = await makeProof(fx, withExpiry(expiry));

            await time.increaseTo(expiry + 1);

            for (const ep of entryPoints) {
                await expect(ep.call(fx.lightNode, bytes), ep.name)
                    .to.be.revertedWithCustomError(fx.lightNode, "LightNodeV3_Signature_Expired")
                    .withArgs(expiry, anyValue);
            }
        });

        it("reports the expiry and the current time in the error", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            const bytes = await makeProof(fx, withExpiry(expiry));

            await time.increaseTo(expiry + 50);
            const now = await time.latest();

            await expect(fx.lightNode["verifyProofData(bytes)"](bytes))
                .to.be.revertedWithCustomError(fx.lightNode, "LightNodeV3_Signature_Expired")
                .withArgs(expiry, now);
        });

        it("an expiry of 1 (long in the past) is expired", async function () {
            const fx = await loadFixture(deployFixture);
            const bytes = await makeProof(fx, withExpiry(1));
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes))
                .to.be.revertedWithCustomError(fx.lightNode, "LightNodeV3_Signature_Expired")
                .withArgs(1, anyValue);
        });

        it("is checked before the signatures (garbage signatures still report expiry)", async function () {
            const fx = await loadFixture(deployFixture);
            const bytes = await makeProof(fx, withExpiry(1), { signatures: [] });
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                fx.lightNode,
                "LightNodeV3_Signature_Expired"
            );
        });

        it("pause takes precedence over expiry", async function () {
            const fx = await loadFixture(deployFixture);
            const bytes = await makeProof(fx, withExpiry(1));
            await fx.lightNode.pause();
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWith("Pausable: paused");
        });
    });

    describe("boundary", function () {
        it("is valid while block.timestamp == expiry and expired one second later", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 1000;
            const bytes = await makeProof(fx, withExpiry(expiry));

            await time.increaseTo(expiry - 1);
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;

            await time.increaseTo(expiry);
            expect(await time.latest()).to.equal(expiry);
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;

            await time.increaseTo(expiry + 1);
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                fx.lightNode,
                "LightNodeV3_Signature_Expired"
            );
        });
    });

    describe("expiry is part of the signed message", function () {
        it("removing the expiry from an expired proof invalidates the signatures", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            const signed = withExpiry(expiry);
            await time.increaseTo(expiry + 1);

            // signers signed `signed`, an attacker submits the same block position without expiry
            const bytes = await makeProof(fx, lowPosition(), { signedPosition: signed });
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("extending the expiry invalidates the signatures", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            await time.increaseTo(expiry + 1);

            const bytes = await makeProof(fx, withExpiry(expiry + 100 * DAY), {
                signedPosition: withExpiry(expiry),
            });
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("a proof signed without expiry cannot be given an expiry", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;

            const bytes = await makeProof(fx, withExpiry(expiry), { signedPosition: lowPosition() });
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("proofs for the same block with different expiries are independent", async function () {
            const fx = await loadFixture(deployFixture);
            const now = await time.latest();
            const shortLived = await makeProof(fx, withExpiry(now + 100));
            const longLived = await makeProof(fx, withExpiry(now + 100 * DAY));

            await time.increaseTo(now + 200);
            await expect(fx.lightNode["verifyProofData(bytes)"](shortLived)).to.be.revertedWithCustomError(
                fx.lightNode,
                "LightNodeV3_Signature_Expired"
            );
            expect((await fx.lightNode["verifyProofData(bytes)"](longLived)).success).to.be.true;
        });
    });

    describe("position validation", function () {
        it("a position with an empty low part is invalid, even with an expiry", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;

            for (const position of [BigNumber.from(0), withExpiry(expiry, BigNumber.from(0)), BigNumber.from(2).pow(128)]) {
                const bytes = await makeProof(fx, position);
                await expect(fx.lightNode["verifyProofData(bytes)"](bytes), position.toHexString()).to.be.revertedWithCustomError(
                    fx.lightNode,
                    "LightNodeV3_Invalid_Position"
                );
            }
        });

        it("a position above 2^128 is now valid when its low part is set (the high part is the expiry)", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const position = withExpiry(expiry);
            expect(position.gt(BigNumber.from(2).pow(128))).to.be.true;

            const bytes = await makeProof(fx, position);
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;
        });

        it("quorum and signer checks are unchanged for proofs with an expiry", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const position = withExpiry(expiry);

            // only one signature, quorum is 2
            const hash = keccak256(
                ethers.utils.solidityPack(
                    ["bytes32", "bytes32", "uint256", "uint256"],
                    [keccak256(ENCODE_LOG), fx.version, position, chainId]
                )
            );
            const s1 = await fx.addr1.signMessage(ethers.utils.arrayify(hash));
            const oneSig = await makeProof(fx, position, { signatures: [s1] });
            await expect(fx.lightNode["verifyProofData(bytes)"](oneSig)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_QuorumNotReached"
            );

            // the same signer twice
            const twice = await makeProof(fx, position, { signatures: [s1, s1] });
            await expect(fx.lightNode["verifyProofData(bytes)"](twice)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_SignerAlreadySigned"
            );

            // a signer outside of the multisig
            const [, , , outsider] = await ethers.getSigners();
            const so = await outsider.signMessage(ethers.utils.arrayify(hash));
            const outsiderProof = await makeProof(fx, position, { signatures: [s1, so] });
            await expect(fx.lightNode["verifyProofData(bytes)"](outsiderProof)).to.be.revertedWithCustomError(
                fx.lightNode,
                "ECDSAMultisig_RecoveredSignerNotAuthorized"
            );
        });

        it("a multisig update (new version) invalidates proofs signed with an expiry", async function () {
            const fx = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const bytes = await makeProof(fx, withExpiry(expiry));
            expect((await fx.lightNode["verifyProofData(bytes)"](bytes)).success).to.be.true;

            await fx.lightNode.updateMultisig(1, [fx.addr1.address]);
            await expect(fx.lightNode["verifyProofData(bytes)"](bytes)).to.be.reverted;
        });
    });
});
