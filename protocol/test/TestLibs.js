const { ethers } = require("hardhat");
const { expect } = require("chai");
const { expectRevert } = require("./helpers/utils");

const D32 = "0x" + "aa".repeat(32);

const { RLP, hexlify, hexConcat, toUtf8Bytes, hexZeroPad } = ethers.utils;

const ADDR1 = "0x1111111111111111111111111111111111111111";
const ADDR2 = "0x2222222222222222222222222222222222222222";
const T1 = ethers.utils.id("Transfer(address,address,uint256)");
const T2 = ethers.utils.id("Other(uint256)");

// [address, [topics], data]
const log = (addr, topics, data) => [addr, topics, data];
const logsOf = (...l) => RLP.encode(l);
const receipt = (logs) => RLP.encode(["0x01", "0x5208", "0x" + "00".repeat(256), logs]);

describe("LogDecode", function () {
    let h;
    const L0 = log(ADDR1, [T1, T2], D32);
    const L1 = log(ADDR2, [T2], "0x" + "cd".repeat(40));
    const L2 = log(ADDR1, [T2], "0x" + "ab".repeat(100)); // long data (> 55 bytes, long RLP prefix)

    let rlp;
    before(async () => {
        h = await (await ethers.getContractFactory("LogDecodeHarness")).deploy();
        rlp = await (await ethers.getContractFactory("RLPReaderHarness")).deploy();
    });

    const expectLog = (l, a, topics, data) => {
        expect(l.addr).to.equal(ethers.utils.getAddress(a));
        expect(l.topics).to.deep.equal(topics);
        expect(l.data).to.equal(data);
    };

    it("decodeTxLogs decodes every log", async () => {
        const r = await h.decodeTxLogs(logsOf(L0, L1, L2));
        expect(r.length).to.equal(3);
        expectLog(r[0], ADDR1, [T1, T2], D32);
        expectLog(r[1], ADDR2, [T2], L1[2]);
        expectLog(r[2], ADDR1, [T2], L2[2]);
    });

    it("decoded data is not corrupted by a following topic-less log (no memory aliasing)", async () => {
        const short = log(ADDR1, [T1], "0xaabbcc");
        const empty = log(ADDR2, [], "0x");
        const r = await h.decodeTxLogs(logsOf(short, empty));
        expect(r[0].data).to.equal("0xaabbcc");
        expect(r[1].data).to.equal("0x");
        const full = await h.decodeTxLogs(logsOf(log(ADDR1, [], D32), log(ADDR2, [], "0x"), log(ADDR1, [], D32)));
        expect(full[0].data).to.equal(D32);
        expect(full[2].data).to.equal(D32);
    });

    it("decodeTxLogs of an empty list returns empty array", async () => {
        expect((await h.decodeTxLogs(logsOf())).length).to.equal(0);
    });

    it("decodeTxLog picks the log by index", async () => {
        const logs = logsOf(L0, L1, L2);
        expectLog(await h.decodeTxLog(logs, 0), ADDR1, [T1, T2], D32);
        expectLog(await h.decodeTxLog(logs, 1), ADDR2, [T2], L1[2]);
        expectLog(await h.decodeTxLog(logs, 2), ADDR1, [T2], L2[2]);
    });

    it("decodeTxLog reverts for an index out of range", async () => {
        await expectRevert(h.decodeTxLog(logsOf(L0, L1), 2));
        await expectRevert(h.decodeTxLog(logsOf(L0, L1), 50));
    });

    it("decodeLogRlp decodes a single log", async () => {
        expectLog(await h.decodeLogRlp(RLP.encode(L0)), ADDR1, [T1, T2], D32);
    });

    it("rejects a log with fewer than 3 fields", async () => {
        await expectRevert(h.decodeLogRlp(RLP.encode([ADDR1, [T1]])), "log length to low");
    });

    it("rejects a topic that is not 32 bytes", async () => {
        await expectRevert(h.decodeLogRlp(RLP.encode([ADDR1, ["0x1234"], "0x"])), "item is not bytes32");
    });

    it("rejects an address that is not 20 bytes", async () => {
        await expectRevert(h.decodeLogRlp(RLP.encode(["0x1234", [T1], "0x"])), "item len is not 21");
    });

    describe("RLP edge cases", () => {
        const addr = "94" + "11".repeat(20);

        it("still accepts a well formed long (> 55 bytes) list", async () => {
            const l = log(ADDR1, [T1, T2], "0x" + "ab".repeat(300));
            expectLog(await h.decodeLogRlp(RLP.encode(l)), ADDR1, [T1, T2], l[2]);
        });

        it("tolerates trailing bytes after the item (downstream light clients rely on it)", async () => {
            // lightclients/matic Verify._getValidatorsAfterCanCun passes an RLP list followed by seal bytes
            const validators = "0x" + "ab".repeat(80);
            const buf = hexConcat([RLP.encode([validators, "0x01"]), "0x" + "cd".repeat(32)]);
            expect(await rlp.firstItem(buf)).to.equal(validators);
            const good = RLP.encode([ADDR1, [], "0x1234"]);
            expectLog(await h.decodeLogRlp(hexConcat([good, "0xdeadbeef"])), ADDR1, [], "0x1234");
        });

        it("[known limitation] RLPReader does not enforce canonical encoding or item types", async () => {
            // the reader is only used on hash-authenticated data, so these are accepted:
            const list33 = "0xe0" + "22".repeat(32); // a LIST read as uint / bytes32
            expect(await rlp.toUint(list33)).to.equal("0x" + "22".repeat(32));
            expect(await rlp.toBytes32(list33)).to.equal("0x" + "22".repeat(32));
            expect(await rlp.toUint("0x8200ff")).to.equal(255); // leading zero byte
            expect(await rlp.toBytes("0x8100")).to.equal("0x00"); // single byte < 0x80 in long form
            // non canonical long-form string of 31 bytes decodes shifted by the length byte
            expect(await rlp.toBytes32("0xb81f" + "33".repeat(31))).to.equal("0x1f" + "33".repeat(31));
        });

        it("RLPReader basic typed accessors", async () => {
            expect(await rlp.toUint("0x80")).to.equal(0);
            expect(await rlp.toUint("0x05")).to.equal(5);
            expect(await rlp.toBoolean("0x01")).to.equal(true);
            expect(await rlp.toBoolean("0x80")).to.equal(false);
            expect(await rlp.toBoolean("0x00")).to.equal(false);
            expect(await rlp.toAddress("0x94" + "ab".repeat(20))).to.equal(ethers.utils.getAddress("0x" + "ab".repeat(20)));
            await expectRevert(rlp.toAddress("0x93" + "ab".repeat(19)), "item len is not 21");
            expect(await rlp.isList("0xc0")).to.equal(true);
            expect(await rlp.isList("0x80")).to.equal(false);
            expect(await rlp.itemByIndex("0xc3010203", 2)).to.equal("0x03");
            await expectRevert(rlp.itemByIndex("0xc3010203", 5), "RLP item overflow");
        });

        it("[known limitation] a declared length beyond the buffer is not rejected (reads past the buffer)", async () => {
            // only safe because callers feed hash-authenticated data into the reader
            const body = addr + "c0" + "b840aabbccdd"; // data claims 64 bytes, only 4 present
            const raw = "0x" + (0xc0 + body.length / 2).toString(16) + body;
            const r = await h.decodeLogRlp(raw);
            expect((r.data.length - 2) / 2).to.equal(64);
        });
    });

    it("getLogsFromReceipt returns the rlp encoded logs (legacy receipt)", async () => {
        const logs = logsOf(L0, L1);
        expect(await h.getLogsFromReceipt(receipt(JSON.parse(JSON.stringify([L0, L1]))))).to.equal(logs);
    });

    it("getLogsFromTypedReceipt handles legacy (type 0) and typed receipts", async () => {
        const logs = logsOf(L0, L1);
        const legacy = receipt([L0, L1]);
        const typed = hexConcat(["0x02", legacy]);
        expect(await h.getLogsFromTypedReceipt(0, legacy)).to.equal(logs);
        expect(await h.getLogsFromTypedReceipt(2, typed)).to.equal(logs);
    });

    it("decodeTxLogFromReceipt / FromTypedReceipt decode by log index", async () => {
        const legacy = receipt([L0, L1, L2]);
        const typed = hexConcat(["0x02", legacy]);
        expectLog(await h.decodeTxLogFromReceipt(1, legacy), ADDR2, [T2], L1[2]);
        expectLog(await h.decodeTxLogFromTypedReceipt(2, 0, legacy), ADDR1, [T2], L2[2]);
        expectLog(await h.decodeTxLogFromTypedReceipt(0, 2, typed), ADDR1, [T1, T2], D32);
        await expectRevert(h.decodeTxLogFromReceipt(3, legacy));
    });

    it("typed receipt decoded with the wrong type offset fails", async () => {
        const typed = hexConcat(["0x02", receipt([L0])]);
        await expectRevert(h.getLogsFromTypedReceipt(0, typed));
    });

    it("truncated receipt is rejected", async () => {
        const r = receipt([L0, L1]);
        await expectRevert(h.getLogsFromTypedReceipt(2, hexConcat(["0x02", r]).slice(0, 40)));
    });
});

describe("Utils", function () {
    let u;
    before(async () => {
        u = await (await ethers.getContractFactory("UtilsHarness")).deploy();
    });

    it("checkBytes compares by content", async () => {
        expect(await u.checkBytes("0x1234", "0x1234")).to.equal(true);
        expect(await u.checkBytes("0x1234", "0x1235")).to.equal(false);
        expect(await u.checkBytes("0x", "0x")).to.equal(true);
    });

    it("toBytes / fromBytes round trip an address", async () => {
        expect(await u.toBytes(ADDR1)).to.equal(ADDR1);
        expect(await u.fromBytes(ADDR1)).to.equal(ethers.utils.getAddress(ADDR1));
    });

    it("fromBytes takes the first 20 bytes of longer input", async () => {
        expect(await u.fromBytes(ADDR2 + "ff".repeat(12))).to.equal(ethers.utils.getAddress(ADDR2));
    });

    it("splitExtra returns the first 64 bytes and rejects shorter input", async () => {
        const data = "0x" + "ab".repeat(64) + "cd".repeat(10);
        expect(await u.splitExtra(data)).to.equal("0x" + "ab".repeat(64));
        await expectRevert(u.splitExtra("0x" + "ab".repeat(63)), "Invalid extra result type");
    });

    describe("hexStrToBytes", () => {
        const prefix = "0".repeat(64); // first 64 chars are skipped by the function
        const enc = (s) => hexlify(toUtf8Bytes(prefix + s));

        it("decodes lower / upper / mixed case hex", async () => {
            expect(await u.hexStrToBytes(enc("abcdef01"))).to.equal("0xabcdef01");
            expect(await u.hexStrToBytes(enc("ABCDEF01"))).to.equal("0xabcdef01");
            expect(await u.hexStrToBytes(enc("aBcD09"))).to.equal("0xabcd09");
        });

        it("rejects odd length, too short and non hex characters", async () => {
            await expectRevert(u.hexStrToBytes(enc("abc")), "invalid input");
            await expectRevert(u.hexStrToBytes(hexlify(toUtf8Bytes("ab"))), "invalid input");
            await expectRevert(u.hexStrToBytes(enc("zz")), "invalid input");
            await expectRevert(u.hexStrToBytes(enc("0g")), "invalid input");
        });
    });

    describe("isValidNearAddress", () => {
        const check = (s) => u.isValidNearAddress(toUtf8Bytes(s));
        it("accepts valid account ids", async () => {
            for (const s of ["alice.near", "a1", "bob_the-builder.testnet", "0123456789", "a".repeat(64)])
                expect(await check(s), s).to.equal(true);
        });

        it("rejects invalid account ids", async () => {
            for (const s of [
                "a", // too short
                "a".repeat(65), // too long
                "Alice.near", // upper case
                "-alice",
                "alice-",
                ".alice",
                "alice.",
                "al..ice",
                "al-_ice",
                "al ice",
                "alice@near",
            ])
                expect(await check(s), s).to.equal(false);
        });
    });

    it("isValidAddress dispatches on the chain type", async () => {
        expect(await u.isValidAddress(ADDR1, 1)).to.equal(true);
        expect(await u.isValidAddress("0x1234", 1)).to.equal(false);
        expect(await u.isValidAddress(toUtf8Bytes("alice.near"), 2)).to.equal(true);
        expect(await u.isValidAddress(toUtf8Bytes("Alice"), 2)).to.equal(false);
        expect(await u.isValidAddress(ADDR1, 0)).to.equal(false);
        expect(await u.isValidAddress(ADDR1, 3)).to.equal(false);
    });
});
