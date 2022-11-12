// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IMOSV2 {
    function transferOutToken(address token, bytes memory to, uint amount, uint toChain) external;
    function transferOutNative(bytes memory to, uint toChain) external payable;
    function depositToken(address token, address to, uint amount) external;
    function depositNative(address to) external payable ;


    event mapTransferOut(bytes token, bytes from, bytes32 orderId,
        uint256 fromChain, uint256 toChain, bytes to, uint256 amount, bytes toChainToken);

    event mapTransferIn(address indexed token, bytes indexed from, bytes32 indexed orderId,
        uint256 fromChain, uint256 toChain, address to, uint256 amount);

    event mapDepositOut(address indexed token, bytes from, bytes32 orderId, uint256 fromChain, uint256 toChain, address to, uint256 amount);


}