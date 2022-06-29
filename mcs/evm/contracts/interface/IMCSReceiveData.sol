// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IMCSReceiveData {
    function receiveCrossChainData(uint fromChain, address from, bytes data) external;
}