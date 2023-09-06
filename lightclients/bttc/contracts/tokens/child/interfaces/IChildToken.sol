// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

interface IChildToken {
    event WithdrawTo(address indexed from, address indexed to, uint256 amount);
    function deposit(address user, bytes calldata depositData) external;
}