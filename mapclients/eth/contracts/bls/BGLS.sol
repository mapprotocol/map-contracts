// SPDX-License-Identifier: UNLICENSED
pragma solidity >0.8.0;

contract BGLS {
    struct G1 {
        uint x;
        uint y;
    }

    G1 g1 = G1(1, 2);

    struct G2 {
        uint xr;
        uint xi;
        uint yr;
        uint yi;
    }

    G2 g2 = G2(
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b
    );

    function modPow(uint base, uint exponent, uint modulus) internal returns (uint) {
        uint[6] memory input = [32, 32, 32, base, exponent, modulus];
        uint[1] memory result;
        assembly {
            if iszero(call(not(0), 0x05, 0, input, 0xc0, result, 0x20)) {
                revert(0, 0)
            }
        }
        return result[0];
    }

    function addPoints(G1 memory a, G1 memory b) public returns (G1 memory) {
        uint[4] memory input = [a.x, a.y, b.x, b.y];
        uint[2] memory result;
        assembly {
            if iszero(call(not(0), 0x06, 0, input, 0x80, result, 0x40)) {
                revert(0, 0)
            }
        }
        return G1(result[0], result[1]);
    }

    function chkBit(bytes memory b, uint x) internal pure returns (bool) {
        return uint(uint8(b[x / 8])) & (uint(1) << (x % 8)) != 0;
    }

    function sumPoints(G1[] memory points, bytes memory indices) public returns (G1 memory) {
        G1 memory acc = G1(0, 0);
        for (uint i = 0; i < points.length; i++) {
            if (chkBit(indices, i)) {
                acc = addPoints(acc, points[i]);
            }
        }
        return G1(acc.x, acc.y);
    }

    // kP
    function scalarMultiply(G1 memory point, uint scalar) public returns (G1 memory) {
        uint[3] memory input = [point.x, point.y, scalar];
        uint[2] memory result;
        assembly {
            if iszero(call(not(0), 0x07, 0, input, 0x60, result, 0x40)) {
                revert(0, 0)
            }
        }
        return G1(result[0], result[1]);
    }

    //returns e(a,x) == e(b,y)
    function pairingCheck(G1 memory a, G2 memory x, G1 memory b, G2 memory y) public returns (bool) {
        uint[12] memory input = [a.x, a.y, x.xi, x.xr, x.yi, x.yr, b.x, prime - b.y, y.xi, y.xr, y.yi, y.yr];
        uint[1] memory result;
        assembly {
            if iszero(call(not(0), 0x08, 0, input, 0x180, result, 0x20)) {
                revert(0, 0)
            }
        }
        return result[0] == 1;
    }


    // compatible with https://github.com/dusk-network/dusk-crypto/blob/master/bls/bls.go#L138-L148
    // which is used in github.com/mapprotocol/atlas
    // https://github.com/mapprotocol/atlas/blob/main/helper/bls/bn256.go#L84-L94
    // todo by long: we might need a better way to hash to G1
    function hashToG1(bytes memory message) public returns (G1 memory) {
        uint h = uint(keccak256(abi.encodePacked(message))) % order;
        return scalarMultiply(g1, h);
    }

    uint prime = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47;
    uint order = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;

    uint pminus = 21888242871839275222246405745257275088696311157297823662689037894645226208582;
    uint pplus = 21888242871839275222246405745257275088696311157297823662689037894645226208584;

    //    function hashToG1(uint[] memory b) internal returns (G1 memory) {
    //        uint x = 0;
    //        G1 memory res;
    //        while (true) {
    //            uint hx = uint(keccak256(abi.encodePacked(b, x))) % prime;
    //            uint px = (modPow(hx, 3, prime) + 3);
    //             y^2 = x^3 % p + 3
    //            if (modPow(px, pminus / 2, prime) == 1) {// determine if px is a quadratic residue, === 1 means yes
    //                 refer to https://mathworld.wolfram.com/QuadraticResidue.html
    //                 prime is a special form of 4k+3,
    //                 where k = 5472060717959818805561601436314318772174077789324455915672259473661306552145
    //                 then x of x^2 = q (mod p) can be solved with x = q^(k+1) mod p, where k = (p - 3) / 4, k + 1 = (p + 1) / 4
    //                uint py = modPow(px, pplus / 4, prime);
    //
    //                res = uint(keccak256(abi.encodePacked(b, uint(255)))) % 2 == 0 ? G1(hx, py) : G1(hx, prime - py);
    //                break;
    //            }
    //            x++;
    //        }
    //        return res;
    //    }

    function checkSignature(bytes memory message, G1 memory sig, G2 memory aggKey) public returns (bool) {
        return pairingCheck(sig, g2, hashToG1(message), aggKey);
    }
}
