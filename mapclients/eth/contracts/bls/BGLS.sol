// SPDX-License-Identifier: UNLICENSED
pragma solidity >0.8.0;

contract BGLS {
    struct G1 {
        uint x;
        uint y;
    }

    struct G2 {
        uint xr;
        uint xi;
        uint yr;
        uint yi;
    }

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

    function checkSignature(bytes memory message, G1 memory sig, G2 memory aggKey) public returns (bool) {
        return pairingCheck(sig, g2, hashToG1(message), aggKey);
    }
}
