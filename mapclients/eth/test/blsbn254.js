"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
exports.__esModule = true;
exports.bigToHex = exports.randHex = exports.randG2 = exports.randG1 = exports.randFr = exports.newG2 = exports.newG1 = exports.compressSignature = exports.compressPubkey = exports.aggreagate = exports.verify = exports.sign = exports.newKeyPair = exports.g2ToHex = exports.g2ToBN = exports.g2ToCompressed = exports.g1ToHex = exports.g1ToBN = exports.g1ToCompressed = exports.signOfG2 = exports.signOfG1 = exports.g2Mul = exports.g1Mul = exports.g2 = exports.g1 = exports.mclToHex = exports.hashToG1 = exports.init = exports.ORDER = exports.PRIME = void 0;
var ethers_1 = require("ethers");
var mcl = require('mcl-wasm');
exports.PRIME = ethers_1.BigNumber.from('0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47');
exports.ORDER = ethers_1.BigNumber.from('0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001');
function init() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, mcl.init(mcl.BN_SNARK1)];
                case 1:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    });
}
exports.init = init;
function hashToG1(msg) {
    if (!ethers_1.ethers.utils.isHexString(msg)) {
        throw new Error('message is expected to be hex string');
    }
    var _msg = Uint8Array.from(Buffer.from(msg.slice(2), 'hex'));
    var hash = ethers_1.ethers.utils.solidityKeccak256(["bytes"], [_msg]);
    var h = ethers_1.BigNumber.from(hash).mod(exports.ORDER);
    var e1 = new mcl.Fr();
    e1.setStr(h.toString());
    var p = mcl.mul(g1(), e1);
    p.normalize();
    return p;
}
exports.hashToG1 = hashToG1;
function mclToHex(p, prefix) {
    if (prefix === void 0) { prefix = true; }
    var arr = p.serialize();
    var s = '';
    for (var i = arr.length - 1; i >= 0; i--) {
        s += ('0' + arr[i].toString(16)).slice(-2);
    }
    return prefix ? '0x' + s : s;
}
exports.mclToHex = mclToHex;
function g1() {
    var g1 = new mcl.G1();
    g1.setStr('1 0x01 0x02', 16);
    return g1;
}
exports.g1 = g1;
function g2() {
    var g2 = new mcl.G2();
    g2.setStr('1 0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed 0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2 0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa 0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b');
    return g2;
}
exports.g2 = g2;
function g1Mul(k, p) {
    return mcl.mul(p, k);
}
exports.g1Mul = g1Mul;
function g2Mul(k, p) {
    return mcl.mul(p, k);
}
exports.g2Mul = g2Mul;
function signOfG1(p) {
    var y = ethers_1.BigNumber.from(mclToHex(p.getY()));
    var ONE = ethers_1.BigNumber.from(1);
    return y.and(ONE).eq(ONE);
}
exports.signOfG1 = signOfG1;
function signOfG2(p) {
    p.normalize();
    var y = mclToHex(p.getY(), false);
    var ONE = ethers_1.BigNumber.from(1);
    return ethers_1.BigNumber.from('0x' + y.slice(64))
        .and(ONE)
        .eq(ONE);
}
exports.signOfG2 = signOfG2;
function g1ToCompressed(p) {
    var MASK = ethers_1.BigNumber.from('0x8000000000000000000000000000000000000000000000000000000000000000');
    p.normalize();
    if (signOfG1(p)) {
        var x = ethers_1.BigNumber.from(mclToHex(p.getX()));
        var masked = x.or(MASK);
        return bigToHex(masked);
    }
    else {
        return mclToHex(p.getX());
    }
}
exports.g1ToCompressed = g1ToCompressed;
function g1ToBN(p) {
    p.normalize();
    var x = ethers_1.BigNumber.from(mclToHex(p.getX()));
    var y = ethers_1.BigNumber.from(mclToHex(p.getY()));
    return [x, y];
}
exports.g1ToBN = g1ToBN;
function g1ToHex(p) {
    p.normalize();
    var x = mclToHex(p.getX());
    var y = mclToHex(p.getY());
    return [x, y];
}
exports.g1ToHex = g1ToHex;
function g2ToCompressed(p) {
    var MASK = ethers_1.BigNumber.from('0x8000000000000000000000000000000000000000000000000000000000000000');
    p.normalize();
    var x = mclToHex(p.getX(), false);
    if (signOfG2(p)) {
        var masked = ethers_1.BigNumber.from('0x' + x.slice(64)).or(MASK);
        return [bigToHex(masked), '0x' + x.slice(0, 64)];
    }
    else {
        return ['0x' + x.slice(64), '0x' + x.slice(0, 64)];
    }
}
exports.g2ToCompressed = g2ToCompressed;
function g2ToBN(p) {
    var x = mclToHex(p.getX(), false);
    var y = mclToHex(p.getY(), false);
    return [
        ethers_1.BigNumber.from('0x' + x.slice(64)),
        ethers_1.BigNumber.from('0x' + x.slice(0, 64)),
        ethers_1.BigNumber.from('0x' + y.slice(64)),
        ethers_1.BigNumber.from('0x' + y.slice(0, 64)),
    ];
}
exports.g2ToBN = g2ToBN;
function g2ToHex(p) {
    p.normalize();
    var x = mclToHex(p.getX(), false);
    var y = mclToHex(p.getY(), false);
    return ['0x' + x.slice(64), '0x' + x.slice(0, 64), '0x' + y.slice(64), '0x' + y.slice(0, 64)];
}
exports.g2ToHex = g2ToHex;
function newKeyPair() {
    var secret = randFr();
    var pubkey = mcl.mul(g2(), secret);
    pubkey.normalize();
    return { pubkey: pubkey, secret: secret };
}
exports.newKeyPair = newKeyPair;
function sign(message, secret) {
    var M = hashToG1(message);
    var signature = mcl.mul(M, secret);
    signature.normalize();
    return { signature: signature, M: M };
}
exports.sign = sign;
function verify(message, pubkey, signature) {
    var M = hashToG1(message);
    var e1 = mcl.pairing(M, pubkey);
    var e2 = mcl.pairing(signature, g2());
    return e1.isEqual(e2);
}
exports.verify = verify;
function aggreagate(acc, other) {
    var _acc = mcl.add(acc, other);
    _acc.normalize();
    return _acc;
}
exports.aggreagate = aggreagate;
function compressPubkey(p) {
    return g2ToCompressed(p);
}
exports.compressPubkey = compressPubkey;
function compressSignature(p) {
    return g1ToCompressed(p);
}
exports.compressSignature = compressSignature;
function newG1() {
    return new mcl.G1();
}
exports.newG1 = newG1;
function newG2() {
    return new mcl.G2();
}
exports.newG2 = newG2;
function randFr() {
    var r = randHex(12);
    var fr = new mcl.Fr();
    fr.setHashOf(r);
    return fr;
}
exports.randFr = randFr;
function randG1() {
    var p = mcl.mul(g1(), randFr());
    p.normalize();
    return p;
}
exports.randG1 = randG1;
function randG2() {
    var p = mcl.mul(g2(), randFr());
    p.normalize();
    return p;
}
exports.randG2 = randG2;
function randHex(n) {
    return ethers_1.ethers.utils.hexlify(ethers_1.ethers.utils.randomBytes(n));
}
exports.randHex = randHex;
function bigToHex(n) {
    return ethers_1.ethers.utils.hexZeroPad(n.toHexString(), 32);
}
exports.bigToHex = bigToHex;
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
