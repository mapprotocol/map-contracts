import { loadFixture, time } from "@nomicfoundation/hardhat-network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import { BigNumber } from "ethers";
import { keccak256 } from "ethers/lib/utils";

const chainId = 137;
const receiptRoot = "0x9d1a63e744550eebbb4d141e5b77c13cb1c21f40fb4f124bb9f161cea166b8ff";

// position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
function withExpiry(expiry: number | BigNumber, low: BigNumber): BigNumber {
    return BigNumber.from(expiry).shl(128).or(low);
}

describe("OracleV3 position with signature expiry", function () {
    async function deployFixture() {
        const [wallet, addr1, addr2, addr3] = await ethers.getSigners();

        const Oracle = await ethers.getContractFactory("contracts/v3/OracleV3.sol:OracleV3");
        const impl = await Oracle.deploy();
        await impl.deployed();

        const OracleProxy = await ethers.getContractFactory("OracleProxy");
        const initData = Oracle.interface.encodeFunctionData("initialize", [wallet.address]);
        const proxy = await OracleProxy.deploy(impl.address, initData);
        await proxy.deployed();

        const oracle = Oracle.attach(proxy.address);
        await oracle.updateMultisig(2, [addr1.address, addr2.address, addr3.address]);
        const info = await oracle.multisigInfo();

        const low: BigNumber = await oracle.encodePosition(12913052, 7, 3, 0);

        const sign = async (signer: any, position: BigNumber, root = receiptRoot) => {
            const hash = keccak256(
                ethers.utils.solidityPack(
                    ["bytes32", "bytes32", "uint256", "uint256"],
                    [root, info.version, position, chainId]
                )
            );
            return signer.signMessage(ethers.utils.arrayify(hash));
        };

        return { oracle, version: info.version as string, low, sign, wallet, addr1, addr2, addr3 };
    }

    const propose = (oracle: any, signer: any, position: BigNumber, sig: string, root = receiptRoot) =>
        oracle.connect(signer)["propose(uint256,uint256,bytes32,bytes)"](chainId, position, root, sig);

    it("accepts a proposal whose position carries an expiry and reaches the quorum", async function () {
        const { oracle, low, sign, addr1, addr2 } = await loadFixture(deployFixture);
        const position = withExpiry((await time.latest()) + 3600, low);

        const s1 = await sign(addr1, position);
        const s2 = await sign(addr2, position);

        await expect(propose(oracle, addr1, position, s1))
            .to.emit(oracle, "Proposal")
            .withArgs(addr1.address, chainId, position, receiptRoot, s1);
        await expect(propose(oracle, addr2, position, s2)).to.emit(oracle, "Meet");
    });

    it("the signature covers the expiry: a signature without expiry is rejected for a position with expiry", async function () {
        const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
        const position = withExpiry((await time.latest()) + 3600, low);

        const signedWithoutExpiry = await sign(addr1, low);
        await expect(propose(oracle, addr1, position, signedWithoutExpiry)).to.be.reverted;
    });

    it("the same block position with and without expiry (or different expiries) are different proposals", async function () {
        const { oracle, low, version, sign, addr1 } = await loadFixture(deployFixture);
        const now = await time.latest();
        const p1 = withExpiry(now + 3600, low);
        const p2 = withExpiry(now + 7200, low);

        await propose(oracle, addr1, p1, await sign(addr1, p1));

        const isProposed = (p: BigNumber) =>
            oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, p, addr1.address);
        expect(await isProposed(p1)).to.equal(receiptRoot);
        expect(await isProposed(p2)).to.equal(ethers.constants.HashZero);
        expect(await isProposed(low)).to.equal(ethers.constants.HashZero);
    });

    it("signatures of different expiries are collected separately and meet independently", async function () {
        const { oracle, low, version, sign, addr1, addr2 } = await loadFixture(deployFixture);
        const now = await time.latest();
        const p1 = withExpiry(now + 3600, low);
        const p2 = withExpiry(now + 7200, low);

        await propose(oracle, addr1, p1, await sign(addr1, p1));
        await propose(oracle, addr2, p2, await sign(addr2, p2));

        for (const p of [p1, p2]) {
            const info = await oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](chainId, p, receiptRoot, version);
            expect(info.canVerify).to.be.false;
        }

        await expect(propose(oracle, addr2, p1, await sign(addr2, p1))).to.emit(oracle, "Meet");
        const info1 = await oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](chainId, p1, receiptRoot, version);
        const info2 = await oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](chainId, p2, receiptRoot, version);
        expect(info1.canVerify).to.be.true;
        expect(info2.canVerify).to.be.false;
    });

    it("expiry 0 is the same position as before", async function () {
        const { oracle, low, version, sign, addr1 } = await loadFixture(deployFixture);
        expect(withExpiry(0, low)).to.equal(low);

        await propose(oracle, addr1, low, await sign(addr1, low));
        expect(
            await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, low, addr1.address)
        ).to.equal(receiptRoot);
    });

    describe("expired proposals", function () {
        it("propose rejects an already expired position", async function () {
            const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
            const position = withExpiry(1, low); // long expired
            await expect(propose(oracle, addr1, position, await sign(addr1, position)))
                .to.be.revertedWithCustomError(oracle, "proposal_expired")
                .withArgs(1, anyValue);
        });

        it("reports the expiry and the current time in the error", async function () {
            const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            const position = withExpiry(expiry, low);
            const sig = await sign(addr1, position);

            await time.setNextBlockTimestamp(expiry + 5);
            await expect(propose(oracle, addr1, position, sig))
                .to.be.revertedWithCustomError(oracle, "proposal_expired")
                .withArgs(expiry, expiry + 5);
        });

        it("is accepted when block.timestamp == expiry and rejected one second later", async function () {
            const { oracle, low, sign, addr1, addr2 } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 1000;
            const position = withExpiry(expiry, low);

            await time.setNextBlockTimestamp(expiry);
            await expect(propose(oracle, addr1, position, await sign(addr1, position))).to.emit(oracle, "Proposal");

            await time.setNextBlockTimestamp(expiry + 1);
            await expect(
                propose(oracle, addr2, position, await sign(addr2, position))
            ).to.be.revertedWithCustomError(oracle, "proposal_expired");
        });

        it("expiry 0 never expires", async function () {
            const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
            await time.increase(100 * 365 * 24 * 3600);
            await expect(propose(oracle, addr1, low, await sign(addr1, low))).to.emit(oracle, "Proposal");
        });

        it("batchPropose reverts as a whole when one item is expired", async function () {
            const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
            const ok = withExpiry((await time.latest()) + 3600, low);
            const expired = withExpiry(1, await oracle.encodePosition(12913053, 0, 0, 0));

            await expect(
                oracle.connect(addr1).batchPropose([
                    { chainId, position: ok, rootHash: receiptRoot, signature: await sign(addr1, ok) },
                    { chainId, position: expired, rootHash: receiptRoot, signature: await sign(addr1, expired) },
                ])
            ).to.be.revertedWithCustomError(oracle, "proposal_expired");
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,address)"](
                    chainId,
                    (await oracle.multisigInfo()).version,
                    ok,
                    addr1.address
                )
            ).to.equal(ethers.constants.HashZero);
        });

        it("the block number overload of propose takes the expiry as a parameter", async function () {
            const { oracle, sign, addr1, low } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const position = withExpiry(expiry, low);
            const overload = "propose(uint256,uint256,uint256,uint256,uint256,bytes32,bytes)";

            // past expiry is rejected
            await expect(
                oracle.connect(addr1)[overload](chainId, 12913052, 7, 3, 1, receiptRoot, await sign(addr1, withExpiry(1, low)))
            ).to.be.revertedWithCustomError(oracle, "proposal_expired");

            // the record is the same as the one made through the position overload
            await expect(
                oracle.connect(addr1)[overload](chainId, 12913052, 7, 3, expiry, receiptRoot, await sign(addr1, position))
            ).to.emit(oracle, "Proposal").withArgs(addr1.address, chainId, position, receiptRoot, anyValue);
            const version = (await oracle.multisigInfo()).version;
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, position, addr1.address)
            ).to.equal(receiptRoot);
        });

        it("the block number overload with expiry 0 never expires", async function () {
            const { oracle, sign, addr1, low } = await loadFixture(deployFixture);
            await time.increase(100 * 365 * 24 * 3600);
            await expect(
                oracle
                    .connect(addr1)
                    ["propose(uint256,uint256,uint256,uint256,uint256,bytes32,bytes)"](
                        chainId,
                        12913052,
                        7,
                        3,
                        0,
                        receiptRoot,
                        await sign(addr1, low)
                    )
            ).to.emit(oracle, "Proposal");
        });

        it("the block number overloads of isProposed / proposalInfo / recoverProposal address a position with expiry", async function () {
            const { oracle, sign, addr1, low } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const position = withExpiry(expiry, low);
            await propose(oracle, addr1, position, await sign(addr1, position));
            const version = (await oracle.multisigInfo()).version;

            expect(
                await oracle["isProposed(uint256,bytes32,uint256,uint256,uint256,uint256,address)"](
                    chainId,
                    version,
                    12913052,
                    7,
                    3,
                    expiry,
                    addr1.address
                )
            ).to.equal(receiptRoot);
            // the same block without the expiry is a different record
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,uint256,uint256,uint256,address)"](
                    chainId,
                    version,
                    12913052,
                    7,
                    3,
                    0,
                    addr1.address
                )
            ).to.equal(ethers.constants.HashZero);

            const info = await oracle["proposalInfo(uint256,uint256,uint256,uint256,uint256,bytes32,bytes32)"](
                chainId,
                12913052,
                7,
                3,
                expiry,
                receiptRoot,
                ethers.constants.HashZero
            );
            expect(info.singers).to.deep.equal([addr1.address]);

            await oracle
                .connect(addr1)
                ["recoverProposal(uint256,uint256,uint256,uint256,uint256,address,uint256)"](
                    chainId,
                    12913052,
                    7,
                    3,
                    expiry,
                    addr1.address,
                    0
                );
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, position, addr1.address)
            ).to.equal(ethers.constants.HashZero);
        });

        it("a proposal made before the expiry stays readable and recoverable after it", async function () {
            const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            const position = withExpiry(expiry, low);
            await propose(oracle, addr1, position, await sign(addr1, position));

            await time.increaseTo(expiry + 10);
            const version = (await oracle.multisigInfo()).version;
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, position, addr1.address)
            ).to.equal(receiptRoot);
            const info = await oracle["proposalInfo(uint256,uint256,bytes32,bytes32)"](
                chainId,
                position,
                receiptRoot,
                ethers.constants.HashZero
            );
            expect(info.singers).to.deep.equal([addr1.address]);

            await oracle
                .connect(addr1)
                ["recoverProposal(uint256,uint256,address,uint256)"](chainId, position, addr1.address, 0);
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, position, addr1.address)
            ).to.equal(ethers.constants.HashZero);
        });

        it("an expired position can not be re-proposed after it was recovered", async function () {
            const { oracle, low, sign, addr1 } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 100;
            const position = withExpiry(expiry, low);
            const sig = await sign(addr1, position);
            await propose(oracle, addr1, position, sig);
            await oracle
                .connect(addr1)
                ["recoverProposal(uint256,uint256,address,uint256)"](chainId, position, addr1.address, 0);

            await time.increaseTo(expiry + 10);
            await expect(propose(oracle, addr1, position, sig)).to.be.revertedWithCustomError(
                oracle,
                "proposal_expired"
            );
        });
    });

    describe("encodePosition with expiry", function () {
        it("puts the expiry into the high 128 bits above the usual position", async function () {
            const { oracle, low } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const p: BigNumber = await oracle.encodePosition(12913052, 7, 3, expiry);

            expect(p).to.equal(BigNumber.from(expiry).shl(128).or(low));
            expect(p.shr(128)).to.equal(expiry);
            expect(p.mask(128)).to.equal(low);
        });

        it("expiry 0 leaves the position unchanged", async function () {
            const { oracle, low } = await loadFixture(deployFixture);
            expect(await oracle.encodePosition(12913052, 7, 3, 0)).to.equal(low);
        });

        it("accepts the maximum expiry and rejects anything above 2^128 - 1", async function () {
            const { oracle } = await loadFixture(deployFixture);
            const max = BigNumber.from(2).pow(128).sub(1);
            const p: BigNumber = await oracle.encodePosition(1, 0, 0, max);
            expect(p.shr(128)).to.equal(max);
            await expect(oracle.encodePosition(1, 0, 0, max.add(1))).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
        });

        it("keeps the range checks of the other fields", async function () {
            const { oracle } = await loadFixture(deployFixture);
            await expect(oracle.encodePosition(0, 0, 0, 100)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
            await expect(oracle.encodePosition(BigNumber.from(2).pow(64), 0, 0, 100)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
            await expect(oracle.encodePosition(1, 2 ** 32, 0, 100)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
            await expect(oracle.encodePosition(1, 0, 2 ** 32, 100)).to.be.revertedWithCustomError(
                oracle,
                "invalid_proposal_param"
            );
        });

        it("decodePosition returns the expiry together with the other fields", async function () {
            const { oracle } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const p: BigNumber = await oracle.encodePosition(12913052, 7, 3, expiry);
            const d = await oracle.decodePosition(p);
            expect(d.blockNum).to.equal(12913052);
            expect(d.txIndex).to.equal(7);
            expect(d.logIndex).to.equal(3);
            expect(d.expiry).to.equal(expiry);
        });

        it("the result can be proposed through the position overload", async function () {
            const { oracle, sign, addr1 } = await loadFixture(deployFixture);
            const expiry = (await time.latest()) + 3600;
            const position: BigNumber = await oracle.encodePosition(12913052, 7, 3, expiry);
            await expect(propose(oracle, addr1, position, await sign(addr1, position))).to.emit(oracle, "Proposal");
        });
    });

    it("recoverProposal works for a position with expiry", async function () {
        const { oracle, low, version, sign, addr1 } = await loadFixture(deployFixture);
        const position = withExpiry((await time.latest()) + 3600, low);

        await propose(oracle, addr1, position, await sign(addr1, position));
        await oracle
            .connect(addr1)
            ["recoverProposal(uint256,uint256,address,uint256)"](chainId, position, addr1.address, 0);

        expect(
            await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, position, addr1.address)
        ).to.equal(ethers.constants.HashZero);
    });

    it("batchPropose accepts positions with expiry", async function () {
        const { oracle, low, version, sign, addr1 } = await loadFixture(deployFixture);
        const now = await time.latest();
        const p1 = withExpiry(now + 3600, low);
        const p2 = low;

        await oracle.connect(addr1).batchPropose([
            { chainId, position: p1, rootHash: receiptRoot, signature: await sign(addr1, p1) },
            { chainId, position: p2, rootHash: receiptRoot, signature: await sign(addr1, p2) },
        ]);

        for (const p of [p1, p2]) {
            expect(
                await oracle["isProposed(uint256,bytes32,uint256,address)"](chainId, version, p, addr1.address)
            ).to.equal(receiptRoot);
        }
    });

    it("decodePosition ignores the expiry bits", async function () {
        const { oracle, low } = await loadFixture(deployFixture);
        const position = withExpiry((await time.latest()) + 3600, low);

        const decoded = await oracle.decodePosition(position);
        expect(decoded.blockNum).to.equal(12913052);
        expect(decoded.txIndex).to.equal(7);
        expect(decoded.logIndex).to.equal(3);
        expect(decoded.expiry).to.equal(position.shr(128));
    });

    it("rejects a position whose low part is empty, even if an expiry is set", async function () {
        const { oracle, sign, addr1 } = await loadFixture(deployFixture);
        const empty = withExpiry((await time.latest()) + 3600, BigNumber.from(0));

        await expect(propose(oracle, addr1, empty, await sign(addr1, empty))).to.be.revertedWithCustomError(
            oracle,
            "invalid_proposal_param"
        );
        await expect(
            oracle.connect(addr1).batchPropose([
                { chainId, position: empty, rootHash: receiptRoot, signature: await sign(addr1, empty) },
            ])
        ).to.be.revertedWithCustomError(oracle, "invalid_proposal_param");
        await expect(
            oracle.connect(addr1)["recoverProposal(uint256,uint256,address,uint256)"](chainId, empty, addr1.address, 0)
        ).to.be.revertedWithCustomError(oracle, "invalid_proposal_param");
        await expect(oracle.decodePosition(empty)).to.be.revertedWithCustomError(oracle, "invalid_proposal_param");
    });
});
