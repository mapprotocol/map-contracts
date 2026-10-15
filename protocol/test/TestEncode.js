// RLPEncode / LibRLP checked against the reference implementation in ethers.
const { ethers } = require("hardhat");
const { expect } = require("chai");
require("./helpers/utils");

const { RLP, hexlify, hexStripZeros, randomBytes } = ethers.utils;

let seed = 777;
const rnd = (n) => {
    seed = (seed * 1103515245 + 12345) & 0x7fffffff;
    return seed % n;
};
const rbytes = (n) => hexlify(Uint8Array.from({ length: n }, () => rnd(256)));

// byte lengths around every RLP boundary
const LENGTHS = [0, 1, 2, 31, 32, 54, 55, 56, 57, 100, 255, 256, 257, 1000, 65535, 65536];
const uints = [
    0,
    1,
    0x7f,
    0x80,
    0xff,
    0x100,
    0xffff,
    0x10000,
    ethers.BigNumber.from(2).pow(64).sub(1),
    ethers.BigNumber.from(2).pow(128),
    ethers.constants.MaxUint256,
];
const minimal = (x) => ethers.utils.stripZeros(ethers.utils.arrayify(ethers.utils.hexZeroPad(ethers.BigNumber.from(x).toHexString(), 32)));
const refUint = (x) => RLP.encode(minimal(x));

describe("RLPEncode vs ethers RLP", function () {
    this.timeout(300000);
    let e;
    before(async () => {
        e = await (await ethers.getContractFactory("RLPEncodeHarness")).deploy();
    });

    it("encodeBytes for all boundary lengths", async () => {
        for (const n of LENGTHS) {
            const b = rbytes(n);
            expect(await e.encodeBytes(b), `len ${n}`).to.equal(RLP.encode(b));
        }
    });

    it("encodeBytes single byte < 0x80, == 0x80 and > 0x80", async () => {
        for (const b of ["0x00", "0x01", "0x7f", "0x80", "0x81", "0xff"]) {
            expect(await e.encodeBytes(b), b).to.equal(RLP.encode(b));
        }
    });

    it("encodeString", async () => {
        for (const s of ["", "a", "dog", "x".repeat(55), "x".repeat(56), "x".repeat(300)]) {
            expect(await e.encodeString(s)).to.equal(RLP.encode(ethers.utils.toUtf8Bytes(s)));
        }
    });

    it("encodeUint", async () => {
        for (const x of uints) expect(await e.encodeUint(x), x.toString()).to.equal(refUint(x));
    });

    it("encodeBool", async () => {
        expect(await e.encodeBool(true)).to.equal("0x01");
        expect(await e.encodeBool(false)).to.equal("0x80");
    });

    it("encodeAddress", async () => {
        for (let i = 0; i < 10; i++) {
            const a = ethers.utils.getAddress(rbytes(20));
            expect(await e.encodeAddress(a)).to.equal(RLP.encode(a));
        }
        const z = ethers.constants.AddressZero;
        expect(await e.encodeAddress(z)).to.equal(RLP.encode(z));
    });

    it("[known issue] encodeAddress relies on the memory above the free pointer being zero", async () => {
        // the length word of the returned bytes is only partially written; dirty memory makes the
        // length huge and the encoding panics (or would produce garbage)
        const a = ethers.utils.getAddress(rbytes(20));
        let failed = false;
        try {
            await e.encodeAddressAfterDirtyMemory(a);
        } catch (err) {
            failed = true;
        }
        expect(failed).to.equal(true);
    });

    it("encodeAddress several times in one list", async () => {
        const a = ethers.utils.getAddress(rbytes(20));
        const b = ethers.utils.getAddress(rbytes(20));
        expect(await e.encodeAddressesInOneCall(a, 300, b)).to.equal(RLP.encode([a, "0x012c", b]));
    });

    it("encodeList of pre-encoded items, including long lists", async () => {
        for (const count of [0, 1, 2, 10, 60]) {
            const raw = Array.from({ length: count }, () => rbytes(rnd(70)));
            const encoded = raw.map((r) => RLP.encode(r));
            expect(await e.encodeList(encoded), `count ${count}`).to.equal(RLP.encode(raw));
        }
        const big = [rbytes(70000)];
        expect(await e.encodeList(big.map((r) => RLP.encode(r)))).to.equal(RLP.encode(big));
    });

    it("[known limitation] encodeInt of a negative number is the 32 byte two's complement, not valid signed rlp", async () => {
        expect(await e.encodeInt(-1)).to.equal(RLP.encode("0x" + "ff".repeat(32)));
        expect(await e.encodeInt(5)).to.equal(RLP.encode("0x05"));
    });
});

describe("LibRLP vs ethers RLP", function () {
    this.timeout(300000);
    let l;
    before(async () => {
        l = await (await ethers.getContractFactory("LibRLPHarness")).deploy();
    });

    it("encode(uint256)", async () => {
        for (const x of uints) expect(await l.encodeUint(x), x.toString()).to.equal(refUint(x));
    });

    it("encode(bool) / encode(address)", async () => {
        expect(await l.encodeBool(true)).to.equal("0x01");
        expect(await l.encodeBool(false)).to.equal("0x80");
        const a = ethers.utils.getAddress(rbytes(20));
        expect(await l.encodeAddress(a)).to.equal(RLP.encode(a));
    });

    it("encode(bytes) for all boundary lengths", async () => {
        for (const n of LENGTHS) {
            const b = rbytes(n);
            expect(await l.encodeBytes(b), `len ${n}`).to.equal(RLP.encode(b));
        }
        for (const b of ["0x00", "0x7f", "0x80", "0xff"]) expect(await l.encodeBytes(b), b).to.equal(RLP.encode(b));
    });

    it("lists of uints incl. empty and > 55 byte payload", async () => {
        for (const count of [0, 1, 3, 20, 80]) {
            const xs = Array.from({ length: count }, () => rnd(1 << 30));
            const ref = RLP.encode(xs.map((x) => minimal(x)));
            expect(await l.encodeUints(xs), `count ${count}`).to.equal(ref);
        }
    });

    it("nested mixed list", async () => {
        const a = ethers.utils.getAddress(rbytes(20));
        const c = rbytes(70);
        const d = [1, 128, 300, 0];
        const ref = RLP.encode(["0x07", a, c, ["0x01", "0x80", "0x012c", "0x"]]);
        expect(await l.encodeMixed(7, a, c, d)).to.equal(ref);
    });
});
