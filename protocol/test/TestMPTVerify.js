const { ethers } = require("hardhat");
const { expect } = require("chai");
const { makeTrie } = require("./helpers/trie");
const { expectRevert } = require("./helpers/utils");

const { keccak256, hexlify, randomBytes, RLP } = ethers.utils;
const val = (i) => hexlify(randomBytes(40 + i)); // >= 32 bytes, so nodes are always hashed

describe("MPTVerify", function () {
    let mpt;
    before(async () => {
        mpt = await (await ethers.getContractFactory("MPTVerify")).deploy();
        await mpt.deployed();
    });

    // overloads are addressed by full signature
    const verifyHash = (root, hash, key, proof) =>
        mpt["verifyTrieProof(bytes32,bytes32,bytes,bytes[])"](root, hash, key, proof);
    const verifyValue = (root, value, key, proof) => verifyHash(root, keccak256(value), key, proof);
    const verifyNode = (root, key, proof, node) =>
        mpt["verifyTrieProof(bytes32,bytes,bytes[],bytes)"](root, key, proof, node);

    describe("single leaf trie", () => {
        const k = "0x1234";
        const v = val(0);
        let t;
        before(() => (t = makeTrie({ [k]: v })));

        it("accepts a valid proof", async () => {
            expect(await verifyValue(t.root, v, t.key(k), t.proof(k))).to.equal(true);
        });

        it("node overload hashes the node itself", async () => {
            expect(await verifyNode(t.root, t.key(k), t.proof(k), v)).to.equal(true);
            expect(await verifyNode(t.root, t.key(k), t.proof(k), val(1))).to.equal(false);
        });

        it("rejects a wrong value", async () => {
            expect(await verifyValue(t.root, val(5), t.key(k), t.proof(k))).to.equal(false);
        });

        it("rejects a wrong root", async () => {
            expect(await verifyValue(keccak256("0x01"), v, t.key(k), t.proof(k))).to.equal(false);
        });

        it("rejects a different key", async () => {
            const other = "0x1235";
            expect(await verifyValue(t.root, v, makeTrie({ [other]: v }).key(other), t.proof(k))).to.equal(false);
        });

        it("rejects an empty key", async () => {
            expect(await verifyValue(t.root, v, "0x", t.proof(k))).to.equal(false);
        });

        it("rejects an empty proof", async () => {
            expect(await verifyValue(t.root, v, t.key(k), [])).to.equal(false);
        });

        it("rejects a tampered proof node", async () => {
            const p = t.proof(k).slice();
            p[0] = p[0].slice(0, -2) + (p[0].endsWith("00") ? "01" : "00");
            expect(await verifyValue(t.root, v, t.key(k), p)).to.equal(false);
        });
    });

    describe("odd-length key leaf", () => {
        it("accepts the proof (odd HP prefix)", async () => {
            const k = "0x123"; // 3 nibbles -> odd-length hex-prefix
            const v = val(2);
            const t = makeTrie({ [k]: v });
            expect(await verifyValue(t.root, v, t.key(k), t.proof(k))).to.equal(true);
        });
    });

    describe("branch + extension trie", () => {
        const kv = {};
        const keys = [
            "0xa1b2c301",
            "0xa1b2c302", // long shared prefix -> extension + branch
            "0xa1b2d403",
            "0xa2000004",
            "0xb0000005",
            "0xb0000105",
            "0xc7ffff06",
        ];
        keys.forEach((k, i) => (kv[k] = val(i)));
        let t;
        before(() => (t = makeTrie(kv)));

        keys.forEach((k) => {
            it(`accepts proof for key ${k}`, async () => {
                expect(await verifyValue(t.root, kv[k], t.key(k), t.proof(k))).to.equal(true);
            });
        });

        it("rejects value of a sibling key", async () => {
            const [a, b] = keys;
            expect(await verifyValue(t.root, kv[b], t.key(a), t.proof(a))).to.equal(false);
        });

        it("rejects when the proof belongs to another key", async () => {
            const [a, , c] = keys;
            expect(await verifyValue(t.root, kv[a], t.key(a), t.proof(c))).to.equal(false);
        });

        it("rejects a truncated proof (missing leaf)", async () => {
            const k = keys[0];
            const p = t.proof(k);
            expect(await verifyValue(t.root, kv[k], t.key(k), p.slice(0, p.length - 1))).to.equal(false);
        });

        it("rejects a proof with nodes reordered", async () => {
            const k = keys[0];
            const p = t.proof(k).slice().reverse();
            expect(await verifyValue(t.root, kv[k], t.key(k), p)).to.equal(false);
        });

        it("returns false (no revert) when the key ends before the extension path", async () => {
            const proof = t.proof(keys[0]);
            const shortKey = t.key("0xa1");
            expect(await verifyHash(t.root, keccak256(proof[1]), shortKey, proof)).to.equal(false);
            expect(await verifyHash(t.root, keccak256(proof[2]), shortKey, proof)).to.equal(false);
        });
    });

    describe("malformed input", () => {
        it("returns false (no revert) when the key is shorter than the leaf path", async () => {
            const k = "0x1234";
            const v = val(0);
            const t = makeTrie({ [k]: v });
            expect(await verifyValue(t.root, v, t.key("0x12"), t.proof(k))).to.equal(false);
        });

        it("returns false when a key nibble is out of range at a branch", async () => {
            const kv = { "0x1000": val(0), "0x2000": val(1) };
            const t = makeTrie(kv);
            expect(await verifyValue(t.root, kv["0x1000"], "0x1100", t.proof("0x1000"))).to.equal(false);
        });

        it("does not walk into the value slot of a branch for nibble 16", async () => {
            const V = val(0);
            const N = RLP.encode(["0x2012", V]); // leaf for the remaining nibbles [1,2]
            const items = new Array(16).fill("0x");
            items.push(keccak256(N)); // slot 16 = the branch value slot
            const B = RLP.encode(items);
            expect(await verifyValue(keccak256(B), V, "0x100102", [B, N])).to.equal(false);
        });

        it("[known limitation] keys that are a prefix of another key are not supported", async () => {
            // the value of 0x00 lives in the branch value slot behind an extension node; the verifier
            // rejects an extension that ends exactly at the end of the key. Ethereum tries (receipts,
            // transactions, hashed storage / state keys) never contain prefix keys.
            const kv = { "0x00": val(0), "0x0001": val(1) };
            const t = makeTrie(kv);
            expect(await verifyValue(t.root, kv["0x0001"], t.key("0x0001"), t.proof("0x0001"))).to.equal(true);
            expect(await verifyValue(t.root, kv["0x00"], t.key("0x00"), t.proof("0x00"))).to.equal(false);
        });

        it("[known limitation] a proof that walks into an empty branch slot reverts instead of returning false", async () => {
            // absent key: the child reference is the empty string 0x80, toUintStrict requires 33 bytes
            const kv = { "0x1000": val(0), "0x2000": val(1) };
            const t = makeTrie(kv);
            await expectRevert(verifyValue(t.root, val(2), "0x0300", [t.proof("0x1000")[0]]), "item is not uint strict");
        });

        it("[known limitation] a key that ends on a branch with an empty value slot 'proves' the empty value", async () => {
            const kv = { "0x10": val(0), "0x11": val(1), "0x20": val(2) };
            const t = makeTrie(kv);
            const proof = t.proof("0x10").slice(0, 2); // root branch + the branch below nibble 1
            // key [1] is not in the trie, yet the hash of the empty value is accepted for it
            expect(await verifyHash(t.root, keccak256("0x"), "0x01", proof)).to.equal(true);
            expect(await verifyValue(t.root, val(3), "0x01", proof)).to.equal(false);
        });

        it("[known limitation] unused trailing proof nodes after the terminal node are ignored", async () => {
            const k = "0x1234";
            const v = val(0);
            const t = makeTrie({ [k]: v });
            expect(await verifyValue(t.root, v, t.key(k), [...t.proof(k), "0x01", "0x02"])).to.equal(true);
        });

        it("reverts on a proof node that is not a valid RLP list", async () => {
            const k = "0x1234";
            const v = val(0);
            const t = makeTrie({ [k]: v });
            const junk = "0x8101";
            // root must equal the hash of the junk node so the hash check passes and the RLP decode is reached
            await expectRevert(verifyValue(keccak256(junk), v, t.key(k), [junk]));
        });
    });
});
