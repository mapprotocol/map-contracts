// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IMCS {
    function transferIn(uint fromChain, bytes memory receiptProof) external;
//    function transferOut(address toContract, uint toChain, bytes memory data) external;
    function transferOutToken(address token, bytes memory to, uint amount, uint toChain) external;
    function transferOutNative(bytes memory to, uint toChain) external payable;
    function depositOutToken(address token, address from, address to, uint amount) external ;
    function depositOutNative(address from, address to) external payable ;
}