// SPDX-License-Identifier: MIT

pragma solidity >0.8.0;

import "./BGLS.sol";
import '@openzeppelin/contracts/utils/math/SafeMath.sol';
import "@openzeppelin/contracts/access/Ownable.sol";
import "../interface/IBLS.sol";


// weights:
// 100 validator: \sum 67 =  \sum 100 - \sum 33
//                \sum 67 = (\sum 50) + \sum 17
// for i [0, 10), [\sum 0-9, \sum 10-19, ..., \sum 90-99]
// cryptographic method to reduce gas

contract WeightedMultiSig is BGLS,IBLS {
    using SafeMath for uint;
    struct validator {
        G1[] pairKeys; // <-- 100 validators, pubkey G2,   (s, s * g2)   s * g1
        uint[] weights; // voting power
        uint256 threshold; // bft, > 2/3,  if  \sum weights = 100, threshold = 67
        uint256 epoch;
    }

    validator[20] public validators;

    uint256 public maxValidators = 20;

    constructor() {
    }

    function setStateInternal(uint256 _threshold, G1[] memory _pairKeys, uint[] memory _weights, uint256 epoch) public override {
        require(_pairKeys.length == _weights.length, 'mismatch arg');
        uint256 id = getValidatorsId(epoch);
        validator storage v = validators[id];

        for (uint256 i = 0; i < _pairKeys.length; i++) {
            v.pairKeys.push(
                G1({
            x : _pairKeys[i].x,
            y : _pairKeys[i].y
            }));
            v.weights.push(_weights[i]);
        }

        v.threshold = _threshold;
        v.epoch = epoch;
    }


    function upateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint256 epoch, bytes memory bits) public override{
        uint256 idPre = getValidatorsIdPrve(epoch);
        validator memory vPre = validators[idPre];
        uint256 id = getValidatorsId(epoch);
        validator storage v = validators[id];
        v.epoch = epoch;
        uint _weight = 0;
        for (uint256 i = 0; i < vPre.pairKeys.length; i++) {
            if (!chkBit(bits, i)) {
                v.pairKeys.push(
                    G1({
                x : vPre.pairKeys[i].x,
                y : vPre.pairKeys[i].y
                }));
                v.weights.push(vPre.weights[i]);
                _weight = _weight + vPre.weights[i];
            }
        }

        if (_pairKeysAdd.length > 0) {
            for (uint256 i = 0; i < _pairKeysAdd.length; i++) {
                v.pairKeys.push(
                    G1({
                x : _pairKeysAdd[i].x,
                y : _pairKeysAdd[i].y
                }));
                v.weights.push(_weights[i]);
                _weight = _weight + _weights[i];
            }
        }
        v.threshold = _weight*2/3;
    }

    function getValidatorsId(uint256 epoch) public view returns (uint){
        return epoch % maxValidators;
    }

    function getValidatorsIdPrve(uint256 epoch) public view returns (uint){
        uint256 id = getValidatorsId(epoch);
        if (id == 0) {
            return maxValidators - 1;
        } else {
            return id - 1;
        }
    }

    function isQuorum(bytes memory bits, uint[] memory weights, uint256 threshold) public pure returns (bool) {
        uint256 weight = 0;
        for (uint256 i = 0; i < weights.length; i++) {
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
    function checkAggPk(bytes memory bits, G2 memory aggPk, G1[] memory pairKeys) public returns (bool) {
        return pairingCheck(sumPoints(pairKeys, bits), g2, g1, aggPk);
    }

    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    //
    function checkSig(
        bytes memory bits, bytes memory message, G1 memory sig, G2 memory aggPk, uint256 epoch
    ) external override returns (bool) {
        uint256 id = getValidatorsId(epoch);
        validator memory v = validators[id];
        return isQuorum(bits, v.weights, v.threshold)
        && checkAggPk(bits, aggPk, v.pairKeys)
        && checkSignature(message, sig, aggPk);
    }
}
