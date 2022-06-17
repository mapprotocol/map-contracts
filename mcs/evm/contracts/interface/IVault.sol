// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IVault {
    function staking(uint amount) external;

    function stakingTo(uint amount, address to) external;

    function withdraw(uint amount) external;
}