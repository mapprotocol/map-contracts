// SPDX-License-Identifier: MIT

pragma solidity 0.8.21;

import "../lib/Pairing.sol";

contract Verifier {
    using Pairing for *;
    struct VerifyingKey {
        Pairing.G1Point alfa1;
        Pairing.G2Point beta2;
        Pairing.G2Point gamma2;
        Pairing.G2Point delta2;
        Pairing.G1Point[] IC;
    }
    struct Proof {
        Pairing.G1Point A;
        Pairing.G2Point B;
        Pairing.G1Point C;
    }

    function verifyingKey() internal pure returns (VerifyingKey memory vk) {
        vk.alfa1 = Pairing.G1Point(
            16275787255037250005441349990062560900561384207012874618957492782364677014531,
            9805882336454400151963333753793873245494801581010067210587938983260504635014
        );

        vk.beta2 = Pairing.G2Point(
            [
                8399062035120703974792767938413690574738523196010265916495073783635606029253,
                17994576911136970111550391161456780413576625927769300028886272693844141324875
            ],
            [
                13651861243378883644391374854457184084239976863608497821079791068138516465900,
                10726700153543304505409684594257017812560567121837144434894453176870571210983
            ]
        );
        vk.gamma2 = Pairing.G2Point(
            [
                11559732032986387107991004021392285783925812861821192530917403151452391805634,
                10857046999023057135944570762232829481370756359578518086990519993285655852781
            ],
            [
                4082367875863433681332203403145435568316851327593401208105741076214120093531,
                8495653923123431417604973247489272438418190587263600148770280649306958101930
            ]
        );
        vk.delta2 = Pairing.G2Point(
            [
                12001521909828986016370611396318882416236503466302232913217763881851190206613,
                12232874281523303924559732315596921668793401378999152546403177653489332487412
            ],
            [
                4673184379035168019941856770492158656188038564615028702382769928926495395211,
                2785270763203291424244453360855007999774725111773979104098148805641150151647
            ]
        );
        vk.IC = new Pairing.G1Point[](2);

        vk.IC[0] = Pairing.G1Point(
            13213261877083889837185362241669380188278200301038201268069774044982689021079,
            14330639766579746504402251072326272689636886549190511622856049689991869097150
        );

        vk.IC[1] = Pairing.G1Point(
            3265034795046760657213076323706267113956634455864704406798382129080105883620,
            6024981418696286068175449926028126866646542379281561203829063180894921107446
        );
    }

    function verify(uint[] memory input, Proof memory proof) internal view returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.IC.length, "verifier-bad-input");
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field, "verifier-gte-snark-scalar-field");
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.IC[i + 1], input[i]));
        }
        vk_x = Pairing.addition(vk_x, vk.IC[0]);
        if (
            !Pairing.pairingProd4(
                Pairing.negate(proof.A),
                proof.B,
                vk.alfa1,
                vk.beta2,
                vk_x,
                vk.gamma2,
                proof.C,
                vk.delta2
            )
        ) return 1;
        return 0;
    }

    /// return bool true if proof is valid
    function verifyProof(uint[8] memory proofs, uint[] memory inputs) external view returns (bool) {
        Proof memory proof;
        proof.A = Pairing.G1Point(proofs[0], proofs[1]);
        proof.B = Pairing.G2Point(
            [proofs[3], proofs[2]], // proofs[2] + proofs[3] * i
            [proofs[5], proofs[4]]
        );
        proof.C = Pairing.G1Point(proofs[6], proofs[7]);
        if (verify(inputs, proof) == 0) {
            return true;
        } else {
            return false;
        }
    }
}
