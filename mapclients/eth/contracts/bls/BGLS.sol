// SPDX-License-Identifier: MIT

pragma solidity >0.8.0;

import "../interface/IBLSPoint.sol";

contract BGLS is IBLSPoint {
    G1 g1 = G1(1, 2);

    G2 g2 = G2(
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b
    );

    function expMod(uint256 base, uint256 e, uint256 m) private view returns (uint256 result) {
        bool success;
        assembly {
        // define pointer
            let p := mload(0x40)
        // store data assembly-favouring ways
            mstore(p, 0x20)             // Length of Base
            mstore(add(p, 0x20), 0x20)  // Length of Exponent
            mstore(add(p, 0x40), 0x20)  // Length of Modulus
            mstore(add(p, 0x60), base)  // Base
            mstore(add(p, 0x80), e)     // Exponent
            mstore(add(p, 0xa0), m)     // Modulus
        // 0x05           id of precompiled modular exponentiation contract
        // 0xc0 == 192    size of call parameters
        // 0x20 ==  32    size of result
            success := staticcall(gas(), 0x05, p, 0xc0, p, 0x20)
        // data
            result := mload(p)
        }
        require(success, "modular exponentiation failed");
    }

    function addPoints(G1 memory a, G1 memory b) public view returns (G1 memory) {
        uint[4] memory input = [a.x, a.y, b.x, b.y];
        uint[2] memory result;
        bool success = false;
        assembly {
            success := staticcall(gas(), 6, input, 0x80, result, 0x40)
        }
        require(success, "add points fail");
        return G1(result[0], result[1]);
    }

    function chkBit(bytes memory b, uint x) internal pure returns (bool) {
        return uint(uint8(b[x / 8])) & (uint(1) << (x % 8)) != 0;
    }

    function sumPoints(G1[] memory points, bytes memory indices) public view returns (G1 memory) {
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
    function pairingCheck(G1 memory a, G2 memory x, G1 memory b, G2 memory y) public view returns (bool) {
        uint[12] memory input = [a.x, a.y, x.xi, x.xr, x.yi, x.yr, b.x, prime - b.y, y.xi, y.xr, y.yi, y.yr];
        uint[1] memory result;
        bool success = false;
        assembly {
            success := staticcall(gas(), 8, input, 0x180, result, 0x20)
        }
        require(success, "pairing check fail");
        return result[0] == 1;

    }


    // compatible with https://github.com/dusk-network/dusk-crypto/blob/master/bls/bls.go#L138-L148
    // which is used in github.com/mapprotocol/atlas
    // https://github.com/mapprotocol/atlas/blob/main/helper/bls/bn256.go#L84-L94

    uint constant order = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;
    uint constant prime = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47;

    // try-and-increment is an acceptable way to hash to G1, although not ideal
    // todo by long: eip-3068 is a much better way, should explore the implementation
    //    function hashToG1(bytes memory message) public view returns (G1 memory) {
    //        uint x = 0;
    //        G1 memory res;
    //        while (true) {
    //            uint hx = uint(keccak256(abi.encodePacked(message, x))) % prime;
    //            uint px = (expMod(hx, 3, prime) + 3) % prime;
    //            // y^2 = x^3 % p + 3
    //
    //            // determine if px is a quadratic residue, 1 means yes
    //            if (expMod(px, (prime - 1) / 2, prime) == 1) {
    //                // refer to https://mathworld.wolfram.com/QuadraticResidue.html
    //                // prime is a special form of 4k+3,
    //                // then x satisfying x^2 = q (mod p) can be solved by calculating x = q^(k+1) mod p,
    //                // where k = (p - 3) / 4, k + 1 = (p + 1) / 4, then x = q^((p+1)/4) mod p
    //                uint py = expMod(px, (prime + 1) / 4, prime);
    //
    //                res = py <= (prime / 2) ? G1(hx, py) : G1(hx, prime - py);
    //                break;
    //            } else {
    //                x++;
    //            }
    //        }
    //        return res;
    //    }

    function checkSignature(bytes memory message, G1 memory sig, G2 memory aggKey) public view returns (bool) {
        return pairingCheck(sig, g2, hashToG1(message), aggKey);
    }

    // adapted from https://github.com/MadBase/MadNet/blob/v0.5.0/crypto/bn256/solidity/contract/crypto.sol

    // curveB is the constant of the elliptic curve for G1: y^2 == x^3 + curveB, with curveB == 3.
    uint256 constant curveB = 3;

    // baseToG1 constants
    //
    // These are precomputed constants which are independent of t.
    // All of these constants are computed modulo prime.
    //
    // (-1 + sqrt(-3))/2
    uint256 constant HashConst1 = 2203960485148121921418603742825762020974279258880205651966;
    // sqrt(-3)
    uint256 constant HashConst2 = 4407920970296243842837207485651524041948558517760411303933;
    // 1/3
    uint256 constant HashConst3 = 14592161914559516814830937163504850059130874104865215775126025263096817472389;
    // 1 + curveB (curveB == 3)
    uint256 constant HashConst4 = 4;

    // Two256ModP == 2^256 mod prime, used in hashToBase to obtain a more uniform hash value.
    uint256 constant Two256ModP = 6350874878119819312338956282401532409788428879151445726012394534686998597021;

    // pMinus1 == -1 mod prime;
    // this is used in sign0 and all ``negative'' values have this sign value.
    uint256 constant pMinus1 = 21888242871839275222246405745257275088696311157297823662689037894645226208582;

    // pMinus2 == prime - 2, this is the exponent used in finite field inversion.
    uint256 constant pMinus2 = 21888242871839275222246405745257275088696311157297823662689037894645226208581;

    // pMinus1Over2 == (prime - 1) / 2;
    // this is the exponent used in computing the Legendre symbol and
    // is also used in sign0 as the cutoff point between ``positive'' and ``negative'' numbers.
    uint256 constant pMinus1Over2 = 10944121435919637611123202872628637544348155578648911831344518947322613104291;

    // pPlus1Over4 == (prime + 1) / 4, this is the exponent used in computing finite field square roots.
    uint256 constant pPlus1Over4 = 5472060717959818805561601436314318772174077789324455915672259473661306552146;

    // bn256G1Add performs elliptic curve addition on the bn256 curve of Ethereum.
    function bn256G1Add(uint256[4] memory input) private view returns (G1 memory res) {
        // computes P + Q
        // input: 4 values of 256 bit each
        //  *) x-coordinate of point P
        //  *) y-coordinate of point P
        //  *) x-coordinate of point Q
        //  *) y-coordinate of point Q

        bool success;
        uint256[2] memory result;
        assembly {
        // 0x06     id of precompiled bn256Add contract
        // 128      size of call parameters, i.e. 128 bytes total
        // 64       size of call return value,
        //              i.e. 64 bytes / 512 bit for a BN256 curve point
            success := staticcall(gas(), 0x06, input, 128, result, 64)
        }
        require(success, "elliptic curve addition failed");

        res.x = result[0];
        res.y = result[1];
    }

    function bn256G1IsOnCurve(G1 memory point) private pure returns (bool) {
        // check if the provided point is on the bn256 curve (y**2 = x**3 + curveB)
        return mulmod(point.y, point.y, prime) ==
        addmod(
            mulmod(point.x, mulmod(point.x, point.x, prime), prime), curveB, prime
        );
    }

    // safeSigningPoint ensures that the HashToG1 point we are returning
    // is safe to sign; in particular, it is not Infinity (the group identity
    // element) or the standard curve generator (curveGen) or its negation.
    function safeSigningPoint(G1 memory input) public pure returns (bool) {
        return (input.x == 0 || input.x == 1) ? false : true;
    }

    function hashToG1(bytes memory message) public view returns (G1 memory h) {
        uint256 t0 = hashToBase(message, 0x00, 0x01);
        uint256 t1 = hashToBase(message, 0x02, 0x03);

        G1 memory h0 = baseToG1(t0);
        G1 memory h1 = baseToG1(t1);

        // Each BaseToG1 call involves a check that we have a valid curve point.
        // Here, we check that we have a valid curve point after the addition.
        // Again, this is to ensure that even if something strange happens, we
        // will not return an invalid curvepoint.
        h = bn256G1Add([h0.x, h0.y, h1.x, h1.y]);
        require(bn256G1IsOnCurve(h), "Invalid hash point: not on elliptic curve");
        require(safeSigningPoint(h), "Dangerous hash point: not safe for signing");
    }

    // invert computes the multiplicative inverse of t modulo prime.
    // When t == 0, s == 0.
    function invert(uint256 t) private view returns (uint256 s) {
        s = expMod(t, pMinus2, prime);
    }

    // sqrt computes the multiplicative square root of t modulo prime.
    // sqrt does not check that a square root is possible; see legendre.
    function sqrt(uint256 t) private view returns (uint256 s) {
        s = expMod(t, pPlus1Over4, prime);
    }

    // legendre computes the legendre symbol of t with respect to prime.
    // That is, legendre(t) == 1 when a square root of t exists modulo
    // prime, legendre(t) == -1 when a square root of t does not exist
    // modulo prime, and legendre(t) == 0 when t == 0 mod prime.
    function legendre(uint256 t) private view returns (int256 chi) {
        uint256 s = expMod(t, pMinus1Over2, prime);
        chi = s != 0 ? (2 * int256(s & 1) - 1) : int(0);
    }

    // neg computes the additive inverse (the negative) modulo prime.
    function neg(uint256 t) private pure returns (uint256 s) {
        s = t == 0 ? 0 : prime - t;
    }

    // sign0 computes the sign of a finite field element.
    // sign0 is used instead of legendre in baseToG1 from the suggestion of WB 2019.
    function sign0(uint256 t) public pure returns (uint256 s) {
        s = 1;
        if (t > pMinus1Over2) {
            s = pMinus1;
        }
    }

    // hashToBase takes in a byte slice message and bytes c0 and c1 for
    // domain separation. The idea is that we treat keccak256 as a random
    // oracle which outputs uint256. The problem is that we want to hash modulo
    // prime (p, a prime number). Just using uint256 mod p will lead
    // to bias in the distribution. In particular, there is bias towards the
    // lower 5% of the numbers in [0, prime). The 1-norm error between
    // s0 mod p and a uniform distribution is ~ 1/4. By itself, this 1-norm
    // error is not too enlightening, but continue reading, as we will compare
    // it with another distribution that has much smaller 1-norm error.
    //
    // To obtain a better distribution with less bias, we take 2 uint256 hash
    // outputs (using c0 and c1 for domain separation so the hashes are
    // independent) and concatenate them to form a ``uint512''. Of course,
    // this is not possible in practice, so we view the combined output as
    //
    //      x == s0*2^256 + s1.
    //
    // This implies that x (combined from s0 and s1 in this way) is a
    // 512-bit uint. If s0 and s1 are uniformly distributed modulo 2^256,
    // then x is uniformly distributed modulo 2^512. We now want to reduce
    // this modulo prime (p). This is done as follows:
    //
    //      x mod p == [(s0 mod p)*(2^256 mod p)] + s1 mod p.
    //
    // This allows us easily compute the result without needing to implement
    // higher precision. The 1-norm error between x mod p and a uniform
    // distribution is ~1e-77. This is a *significant* improvement from s0 mod p.
    // For all practical purposes, there is no difference from a
    // uniform distribution
    function hashToBase(bytes memory message, bytes1 c0, bytes1 c1) internal pure returns (uint256 t) {
        uint256 s0 = uint256(keccak256(abi.encodePacked(c0, message)));
        uint256 s1 = uint256(keccak256(abi.encodePacked(c1, message)));
        t = addmod(mulmod(s0, Two256ModP, prime), s1, prime);
    }

    function baseToG1(uint256 t) internal view returns (G1 memory h) {
        // ap1 and ap2 are temporary variables, originally named to represent
        // alpha part 1 and alpha part 2. Now they are somewhat general purpose
        // variables due to using too many variables on stack.
        uint256 ap1;
        uint256 ap2;

        // One of the main constants variables to form x1, x2, and x3
        // is alpha, which has the following definition:
        //
        //      alpha == (ap1*ap2)^(-1)
        //            == [t^2*(t^2 + h4)]^(-1)
        //
        //      ap1 == t^2
        //      ap2 == t^2 + h4
        //      h4  == HashConst4
        //
        // Defining alpha helps decrease the calls to expMod,
        // which is the most expensive operation we do.
        uint256 alpha;
        ap1 = mulmod(t, t, prime);
        ap2 = addmod(ap1, HashConst4, prime);
        alpha = mulmod(ap1, ap2, prime);
        alpha = invert(alpha);

        // Another important constant which is used when computing x3 is tmp,
        // which has the following definition:
        //
        //      tmp == (t^2 + h4)^3
        //          == ap2^3
        //
        //      h4  == HashConst4
        //
        // This is cheap to compute because ap2 has not changed
        uint256 tmp;
        tmp = mulmod(ap2, ap2, prime);
        tmp = mulmod(tmp, ap2, prime);

        // When computing x1, we need to compute t^4. ap1 will be the
        // temporary variable which stores this value now:
        //
        // Previous definition:
        //      ap1 == t^2
        //
        // Current definition:
        //      ap1 == t^4
        ap1 = mulmod(ap1, ap1, prime);

        // One of the potential x-coordinates of our elliptic curve point:
        //
        //      x1 == h1 - h2*t^4*alpha
        //         == h1 - h2*ap1*alpha
        //
        //      ap1 == t^4 (note previous assignment)
        //      h1  == HashConst1
        //      h2  == HashConst2
        //
        // When t == 0, x1 is a valid x-coordinate of a point on the elliptic
        // curve, so we need no exceptions; this is different than the original
        // Fouque and Tibouchi 2012 paper. This comes from the fact that
        // 0^(-1) == 0 mod p, as we use expMod for inversion.
        uint256 x1;
        x1 = mulmod(HashConst2, ap1, prime);
        x1 = mulmod(x1, alpha, prime);
        x1 = neg(x1);
        x1 = addmod(x1, HashConst1, prime);

        // One of the potential x-coordinates of our elliptic curve point:
        //
        //      x2 == -1 - x1
        uint256 x2;
        x2 = addmod(x1, 1, prime);
        x2 = neg(x2);

        // One of the potential x-coordinates of our elliptic curve point:
        //
        //      x3 == 1 - h3*tmp*alpha
        //
        //      h3 == HashConst3
        uint256 x3;
        x3 = mulmod(HashConst3, tmp, prime);
        x3 = mulmod(x3, alpha, prime);
        x3 = neg(x3);
        x3 = addmod(x3, 1, prime);

        // We now focus on determining residue1; if residue1 == 1,
        // then x1 is a valid x-coordinate for a point on E(F_p).
        //
        // When computing residues, the original FT 2012 paper suggests
        // blinding for security. We do not use that suggestion here
        // because of the possibility of a random integer being returned
        // which is 0, which would completely destroy the output.
        // Additionally, computing random numbers on Ethereum is difficult.
        uint256 y;
        y = mulmod(x1, x1, prime);
        y = mulmod(y, x1, prime);
        y = addmod(y, curveB, prime);
        int256 residue1 = legendre(y);

        // We now focus on determining residue2; if residue2 == 1,
        // then x2 is a valid x-coordinate for a point on E(F_p).
        y = mulmod(x2, x2, prime);
        y = mulmod(y, x2, prime);
        y = addmod(y, curveB, prime);
        int256 residue2 = legendre(y);

        // i is the index which gives us the correct x value (x1, x2, or x3)
        int256 i = (residue1 - 1) * (residue2 - 3) / 4 + 1;

        // This is the simplest way to determine which x value is correct
        // but is not secure. If possible, we should improve this.
        uint256 x;
        if (i == 1) {
            x = x1;
        }
        else if (i == 2) {
            x = x2;
        }
        else {
            x = x3;
        }

        // Now that we know x, we compute y
        y = mulmod(x, x, prime);
        y = mulmod(y, x, prime);
        y = addmod(y, curveB, prime);
        y = sqrt(y);

        // We now determine the sign of y based on t; this is a change from
        // the original FT 2012 paper and uses the suggestion from WB 2019.
        //
        // This is done to save computation, as using sign0 reduces the
        // number of calls to expMod from 5 to 4; currently, we call expMod
        // for inversion (alpha), two legendre calls (for residue1 and
        // residue2), and one sqrt call.
        // This change nullifies the proof in FT 2012 that we have a valid
        // hash function. Whether the proof could be slightly modified to
        // compensate for this change is possible but not currently known.
        //
        // (CHG: At the least, I am not sure that the proof holds, nor am I
        // able to see how the proof could potentially be fixed in order
        // for the hash function to be admissible. This is something I plan
        // to look at in the future.)
        //
        // If this is included as a precompile, it may be worth it to ignore
        // the cost savings in order to ensure uniformity of the hash function.
        // Also, we would need to change legendre so that legendre(0) == 1,
        // or else things would fail when t == 0. We could also have a separate
        // function for the sign determination.
        uint256 ySign;
        ySign = sign0(t);
        y = mulmod(y, ySign, prime);

        h.x = x;
        h.y = y;

        // Before returning the value, we check to make sure we have a valid
        // curve point. This ensures we will always have a valid point.
        // From Fouque-Tibouchi 2012, the only way to get an invalid point is
        // when t == 0, but we have already taken care of that to ensure that
        // when t == 0, we still return a valid curve point.
        require(bn256G1IsOnCurve(h), "Invalid point: not on elliptic curve");
    }
}
