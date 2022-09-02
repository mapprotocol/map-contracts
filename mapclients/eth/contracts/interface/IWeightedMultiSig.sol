// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./IBLSPoint.sol";

interface IWeightedMultiSig is IBLSPoint {

    function setStateInternal(uint256 _threshold, G1[] memory _pairKeys, uint[] memory _weights, uint256 epoch) external;

    function upateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint256 epoch, bytes memory bits) external;

    function checkSig(bytes memory bits, bytes memory message, G1 memory sig, G2 memory aggPk, uint256 epoch) view external returns (bool);

    function maxValidators() view external returns(uint256);

}
