// SPDX-License-Identifier: UNLICENSED
pragma solidity >0.8.0;

import "./BGLS.sol";

// weights:
// 100 validator: \sum 67 =  \sum 100 - \sum 33
//                \sum 67 = (\sum 50) + \sum 17
// for i [0, 10), [\sum 0-9, \sum 10-19, ..., \sum 90-99]
// cryptographic method to reduce gas

contract WeightedMultiSig is BGLS {
    G1[] public pairKeys; // <-- 100 validators, pubkey G2,   (s, s * g2)   s * g1
    uint[] public weights; // voting power
    uint public threshold; // bft, > 2/3,  if  \sum weights = 100, threshold = 67

    constructor(uint _threshold, G1[] memory _pairKeys, uint[] memory _weights) {
        setStateInternal(_threshold, _pairKeys, _weights);
    }

    function setStateInternal(uint _threshold, G1[] memory _pairKeys, uint[] memory _weights) internal {
        require(_pairKeys.length == _weights.length, 'mismatch arg');

        for (uint i = 0; i < _pairKeys.length; i++) pairKeys.push(_pairKeys[i]);

        weights = _weights;
        threshold = _threshold;
    }

    function isQuorum(bytes memory bits) public view returns (bool) {
        uint weight = 0;
        for (uint i = 0; i < weights.length; i++) {
            if (chkBit(bits, i)) weight += weights[i];
        }

        return weight >= threshold;
    }

    //---------------------------------------------------------------
    // validator: (s, s*g2), pkG1 = s*g1, pkG2 = s* g2
    // e(pkG1, g2) = e(g1, pkG2)
    //
    // e(s * g1, g2) = e(g1, g2)^s
    // e(g1, s * g2) = e(g1, g2)^s
    //---------------------------------------------------------------
    // 100 validators, pubkey G2,   (s, s * g2)   s * g1
    // aggPk: \sum G2  \in G2,   aggPk2 = s * g2 + t * g2
    // in solidity: \sum G1      aggPk1 = s * g1 + t * g1
    // bits --> who is involved
    // e(aggPk1, g2) == e(g1, aggPk2)
    // e((s+t)*g1, g2) == e(g1, (s+t)*g2)
    // e((s+t)*g1, g2) = e(g1, g2)^(s+t)
    // e(g1, (s+t)*g2) = e(g1, g2)^(s+t)
    //---------------------------------------------------------------
    function checkAggPk(bytes memory bits, G2 memory aggPk) public returns (bool) {
        return pairingCheck(sumPoints(pairKeys, bits), g2, g1, aggPk);
    }

    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    //
    function checkSig(
        bytes memory bits, bytes memory message, G1 memory sig, G2 memory aggPk
    ) public returns (bool) {
        return isQuorum(bits) && checkAggPk(bits, aggPk) && checkSignature(message, sig, aggPk);
    }
}
