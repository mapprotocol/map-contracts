// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IMCSReceiveData {
    function receiveCrossChainData(uint fromChain, address from, bytes memory data) external;
}