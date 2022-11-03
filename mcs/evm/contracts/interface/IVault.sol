// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IVault {

    function stakingTo(uint amount, address to) external;

    function withdraw(uint amount, address to) external;

    function addFee(uint amount) external;

    function getCorrespondQuantity(uint amount) external view returns (uint);
}