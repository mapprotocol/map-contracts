import {BigNumber, ethers} from 'ethers';

const mcl = require('mcl-wasm');

export const PRIME = BigNumber.from('0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47');
export const ORDER = BigNumber.from('0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001');

export type mclG2 = any;
export type mclG1 = any;
export type mclFP = any;
export type mclFR = any;
export type PublicKey = mclG2;
export type SecretKey = mclFR;

export async function init() {
    await mcl.init(mcl.BN_SNARK1);
}

export function hashToG1(msg: string) {
    if (!ethers.utils.isHexString(msg)) {
        throw new Error('message is expected to be hex string');
    }

    const _msg = Uint8Array.from(Buffer.from(msg.slice(2), 'hex'));
    const hash = ethers.utils.solidityKeccak256(["bytes"], [_msg]);

    const h = BigNumber.from(hash).mod(ORDER);

    let e1 = new mcl.Fr();
    e1.setStr(h.toString());
    const p = mcl.mul(g1(), e1);
    p.normalize();

    return p;
}

export function mclToHex(p: mclFP | mclFR, prefix: boolean = true) {
    const arr = p.serialize();
    let s = '';
    for (let i = arr.length - 1; i >= 0; i--) {
        s += ('0' + arr[i].toString(16)).slice(-2);
    }
    return prefix ? '0x' + s : s;
}

export function g1() {
    const g1 = new mcl.G1();
    g1.setStr('1 0x01 0x02', 16);
    return g1;
}

export function g2() {
    const g2 = new mcl.G2();
    g2.setStr(
        '1 0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed 0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2 0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa 0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b'
    );
    return g2;
}

export function g1Mul(k: mclFR, p: mclG1) {
    return mcl.mul(p, k);
}

export function g2Mul(k: mclFR, p: mclG2) {
    return mcl.mul(p, k);
}

export function signOfG1(p: mclG1): boolean {
    const y = BigNumber.from(mclToHex(p.getY()));
    const ONE = BigNumber.from(1);
    return y.and(ONE).eq(ONE);
}

export function signOfG2(p: mclG2): boolean {
    p.normalize();
    const y = mclToHex(p.getY(), false);
    const ONE = BigNumber.from(1);
    return BigNumber.from('0x' + y.slice(64))
        .and(ONE)
        .eq(ONE);
}

export function g1ToCompressed(p: mclG1) {
    const MASK = BigNumber.from('0x8000000000000000000000000000000000000000000000000000000000000000');
    p.normalize();
    if (signOfG1(p)) {
        const x = BigNumber.from(mclToHex(p.getX()));
        const masked = x.or(MASK);
        return bigToHex(masked);
    } else {
        return mclToHex(p.getX());
    }
}

export function g1ToBN(p: mclG1) {
    p.normalize();
    const x = BigNumber.from(mclToHex(p.getX()));
    const y = BigNumber.from(mclToHex(p.getY()));
    return [x, y];
}

export function g1ToHex(p: mclG1) {
    p.normalize();
    const x = mclToHex(p.getX());
    const y = mclToHex(p.getY());
    return [x, y];
}

export function g2ToCompressed(p: mclG2) {
    const MASK = BigNumber.from('0x8000000000000000000000000000000000000000000000000000000000000000');
    p.normalize();
    const x = mclToHex(p.getX(), false);
    if (signOfG2(p)) {
        const masked = BigNumber.from('0x' + x.slice(64)).or(MASK);
        return [bigToHex(masked), '0x' + x.slice(0, 64)];
    } else {
        return ['0x' + x.slice(64), '0x' + x.slice(0, 64)];
    }
}

export function g2ToBN(p: mclG2) {
    const x = mclToHex(p.getX(), false);
    const y = mclToHex(p.getY(), false);
    return [
        BigNumber.from('0x' + x.slice(64)),
        BigNumber.from('0x' + x.slice(0, 64)),
        BigNumber.from('0x' + y.slice(64)),
        BigNumber.from('0x' + y.slice(0, 64)),
    ];
}

export function g2ToHex(p: mclG2) {
    p.normalize();
    const x = mclToHex(p.getX(), false);
    const y = mclToHex(p.getY(), false);
    return ['0x' + x.slice(64), '0x' + x.slice(0, 64), '0x' + y.slice(64), '0x' + y.slice(0, 64)];
}

export function newKeyPair() {
    const secret = randFr();
    const pubkey = mcl.mul(g2(), secret);
    pubkey.normalize();
    return {pubkey, secret};
}

export function sign(message: string, secret: mclFR) {
    const M = hashToG1(message);
    const signature = mcl.mul(M, secret);
    signature.normalize();
    return {signature, M};
}

export function verify(message: string, pubkey: mclG2, signature: mclG1) {
    const M = hashToG1(message);

    const e1 = mcl.pairing(M, pubkey);
    const e2 = mcl.pairing(signature, g2());

    return e1.isEqual(e2);
}

export function aggreagate(acc: mclG1 | mclG2, other: mclG1 | mclG2) {
    const _acc = mcl.add(acc, other);
    _acc.normalize();
    return _acc;
}

export function compressPubkey(p: mclG2) {
    return g2ToCompressed(p);
}

export function compressSignature(p: mclG1) {
    return g1ToCompressed(p);
}

export function newG1() {
    return new mcl.G1();
}

export function newG2() {
    return new mcl.G2();
}

export function randFr() {
    const r = randHex(12);
    let fr = new mcl.Fr();
    fr.setHashOf(r);
    return fr;
}

export function randG1() {
    const p = mcl.mul(g1(), randFr());
    p.normalize();
    return p;
}

export function randG2() {
    const p = mcl.mul(g2(), randFr());
    p.normalize();
    return p;
}

export function randHex(n: number): string {
    return ethers.utils.hexlify(ethers.utils.randomBytes(n));
}

export function bigToHex(n: BigNumber): string {
    return ethers.utils.hexZeroPad(n.toHexString(), 32);
}

// async function test() {
//     await init();
//
//     console.log('G1', g1ToHex(g1()));
//     console.log('G2', g2ToHex(g2()));
//
//     const message = '0x616263';
//     const p = hashToG1(message);
//     console.log(message, 'hashToG1', g1ToHex(p));
//
//     const keypair = newKeyPair();
//     console.log('sk', bigToHex(BigNumber.from(mclToHex(keypair.secret))));
//     console.log('pk', g2ToHex(keypair.pubkey));
//
//     const signature = sign(message, keypair.secret);
//     console.log('sig', g1ToHex(signature.signature), 'hashToG1', g1ToHex(signature.M));
//
//     const res = verify(message, keypair.pubkey, signature.signature);
//     console.log('verify success', res);
// }
//
// test();

