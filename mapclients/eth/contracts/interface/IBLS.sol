// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./IBLSPoint.sol";


interface IBLS is IBLSPoint {

    function setStateInternal(uint _threshold, G1[] memory _pairKeys, uint[] memory _weights, uint epoch) external;

    function upateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint epoch, bytes memory bits) external;

    function checkSig(bytes memory bits, bytes memory message, G1 memory sig, G2 memory aggPk, uint epoch) external returns (bool);
}