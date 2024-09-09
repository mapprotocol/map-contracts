// This file is part of Darwinia.
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.
//
// Inspired: https://github.com/ralexstokes/deposit-verifier/blob/master/deposit_verifier.sol
//           https://github.com/sigp/milagro_bls

pragma solidity 0.8.17;

import "./Bytes.sol";
import "./Pairing.sol";

library BLS {
    using FP for Fp;
    using G1 for G1Point;
    using G2 for G2Point;
    using Bytes for bytes;

    // Domain Separation Tag for signatures on G2 with a single byte the length of the DST appended
    string private constant DST_G2 = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_+";

    // FastAggregateVerify
    //
    // Verifies an AggregateSignature against a list of PublicKeys.
    // PublicKeys must all be verified via Proof of Possession before running this function.
    // https://tools.ietf.org/html/draft-irtf-cfrg-bls-signature-02#section-3.3.4
    function fast_aggregate_verify(
        bytes[] memory uncompressed_pubkeys,
        bytes32 message,
        bytes memory uncompressed_signature
    ) internal view returns (bool) {
        G1Point memory agg_key = aggregate_pks(uncompressed_pubkeys);

        G2Point memory sign_point = G2.deserialize(uncompressed_signature);
        G2Point memory msg_point = hash_to_curve_g2(message);
        // Faster evaualtion checks e(PK, H) * e(-G1, S) == 1
        return bls_pairing_check(agg_key, msg_point, sign_point);
    }

    function g1_compress(bytes memory uncompressed_pubkey) internal pure returns (bytes memory r) {
          return  G1.serialize(G1.deserialize(uncompressed_pubkey));
    }

    // e(PK, H) * e(-G1, S) == 1
    function bls_pairing_check(G1Point memory pk, G2Point memory h, G2Point memory s) internal view returns (bool) {
        G1Point memory ng1 = G1.negativeP1();
        return Pairing.pairing(pk, h, ng1, s);
    }

    // Aggregate
    //
    // https://tools.ietf.org/html/draft-irtf-cfrg-bls-signature-04#section-2.8
    function aggregate_pks(bytes[] memory pubkeys) internal view returns (G1Point memory) {
        uint len = pubkeys.length;
        require(len > 0, "!pubkeys");
        G1Point memory g1 = G1.deserialize(pubkeys[0]);
        for (uint i = 1; i < len; ) {
            g1 = g1.add(G1.deserialize(pubkeys[i]));
            unchecked { ++i; }
        }
        require(!g1.is_infinity(), "infinity");
        return g1;
    }

    // Hash to Curve
    //
    // Takes a message as input and converts it to a Curve Point
    // https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-09#section-3
    // `MapToCurve` precompile includes `clear_cofactor`
    function hash_to_curve_g2(bytes32 message) internal view returns (G2Point memory) {
        Fp2[2] memory u = hash_to_field_fq2(message);
        G2Point memory q0 = G2.map_to_curve(u[0]);
        G2Point memory q1 = G2.map_to_curve(u[1]);
        return q0.add(q1);
    }


    // Hash To Field - Fp
    //
    // Take a message as bytes and convert it to a Field Point
    // https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-09#section-5.3
    function hash_to_field_fq2(bytes32 message) internal view returns (Fp2[2] memory result) {
        bytes memory uniform_bytes = expand_message_xmd(message);
        result[0] = Fp2(
            reduce_modulo(uniform_bytes.substr(0, 64)),
            reduce_modulo(uniform_bytes.substr(64, 64))
        );
        result[1] = Fp2(
            reduce_modulo(uniform_bytes.substr(128, 64)),
            reduce_modulo(uniform_bytes.substr(192, 64))
        );
    }

    // Expand Message XMD
    //
    // Take a message and convert it to pseudo random bytes of specified length
    // https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-09#section-5.4
    function expand_message_xmd(bytes32 message) internal pure returns (bytes memory) {
        bytes memory b0Input = new bytes(143);
        assembly ("memory-safe") {
            mstore(add(add(b0Input, 0x20), 0x40), message)
        }
        b0Input[96] = 0x01;
        for (uint i = 0; i < 44; ) {
            b0Input[i+99] = bytes(DST_G2)[i];
            unchecked { ++i; }
        }

        bytes32 b0 = sha256(b0Input);

        bytes memory output = new bytes(256);
        bytes32 chunk = sha256(abi.encodePacked(b0, bytes1(0x01), bytes(DST_G2)));
        assembly ("memory-safe") {
            mstore(add(output, 0x20), chunk)
        }
        for (uint i = 2; i < 9; ) {
            bytes32 input;
            assembly ("memory-safe") {
                input := xor(b0, mload(add(output, add(0x20, mul(0x20, sub(i, 2))))))
            }
            chunk = sha256(abi.encodePacked(input, bytes1(uint8(i)), bytes(DST_G2)));
            assembly ("memory-safe") {
                mstore(add(output, add(0x20, mul(0x20, sub(i, 1)))), chunk)
            }
            unchecked { ++i; }
        }

        return output;
    }

    function reduce_modulo(bytes memory base) internal view returns (Fp memory) {
        require(base.length == 64, "!slice");
        Fp memory b = Fp(
            base.slice_to_uint(0, 32),
            base.slice_to_uint(32, 64)
        );
        return b.norm();
    }
}