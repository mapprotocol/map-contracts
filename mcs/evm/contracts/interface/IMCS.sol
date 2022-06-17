// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IMCS {
    function transferIn(uint fromChain, bytes receiptProof) external;
    function transferOut(address toContract, uint toChain, bytes data) external;
    function transferOutToken(address token, bytes to, uint amount, uint toChain) external;
    function transferOutNative(bytes to, uint toChain) external;
    function depositOutToken(address token, address from, bytes to, uint amount) external;
    function depositOutNative(address from, bytes to) external;
}