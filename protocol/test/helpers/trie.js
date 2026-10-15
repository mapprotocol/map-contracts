// Minimal Merkle-Patricia trie builder (ethereum hex-prefix encoding) used to generate proofs for MPTVerify.
// Every value must be >= 32 bytes so that no node is ever embedded (< 32 bytes) in its parent.
const { ethers } = require("hardhat");
const { RLP, keccak256, hexlify } = ethers.utils;

const toNibbles = (hex) => {
    const out = [];
    for (const c of hex.slice(2)) out.push(parseInt(c, 16));
    return out;
};

function hp(nibbles, leaf) {
    const flag = leaf ? 2 : 0;
    const all = nibbles.length % 2 === 1 ? [flag + 1, ...nibbles] : [flag, 0, ...nibbles];
    let hex = "0x";
    for (let i = 0; i < all.length; i += 2) hex += ((all[i] << 4) | all[i + 1]).toString(16).padStart(2, "0");
    return hex;
}

function commonPrefix(lists) {
    let n = 0;
    while (lists.every((l) => l.length > n && l[n] === lists[0][n])) n++;
    return n;
}

// entries: [{ nibbles, value }]
function build(entries) {
    if (entries.length === 1) {
        const e = entries[0];
        const enc = RLP.encode([hp(e.nibbles, true), e.value]);
        return { kind: "leaf", enc, hash: keccak256(enc) };
    }
    const cp = commonPrefix(entries.map((e) => e.nibbles));
    if (cp > 0) {
        const prefix = entries[0].nibbles.slice(0, cp);
        const child = build(entries.map((e) => ({ nibbles: e.nibbles.slice(cp), value: e.value })));
        const enc = RLP.encode([hp(prefix, false), child.hash]);
        return { kind: "ext", enc, hash: keccak256(enc), prefix, child };
    }
    const children = new Array(16).fill(null);
    const items = new Array(17).fill("0x");
    // an entry whose key ends at this branch lives in the value slot (slot 16)
    const here = entries.find((e) => e.nibbles.length === 0);
    if (here) items[16] = here.value;
    entries = entries.filter((e) => e.nibbles.length > 0);
    for (let i = 0; i < 16; i++) {
        const sub = entries
            .filter((e) => e.nibbles[0] === i)
            .map((e) => ({ nibbles: e.nibbles.slice(1), value: e.value }));
        if (sub.length) {
            children[i] = build(sub);
            items[i] = children[i].hash;
        }
    }
    const enc = RLP.encode(items);
    return { kind: "branch", enc, hash: keccak256(enc), children };
}

function prove(node, nibbles) {
    const proof = [node.enc];
    if (node.kind === "branch") {
        if (nibbles.length === 0) return proof; // value slot of the branch
        const c = node.children[nibbles[0]];
        if (c) proof.push(...prove(c, nibbles.slice(1)));
    } else if (node.kind === "ext") {
        proof.push(...prove(node.child, nibbles.slice(node.prefix.length)));
    }
    return proof;
}

/// kv: { keyHex: valueHex }
function makeTrie(kv) {
    const entries = Object.entries(kv).map(([k, v]) => ({ nibbles: toNibbles(k), value: v }));
    const root = build(entries);
    return {
        root: root.hash,
        // the contract takes the key as a byte array holding one nibble per byte
        key: (k) => hexlify(toNibbles(k)),
        proof: (k) => prove(root, toNibbles(k)),
    };
}

module.exports = { makeTrie };
