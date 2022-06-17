// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;


interface IMAPToken {
    function mint(address to, uint256 amount) external;

    function burn(uint256 amount) external;

    function burnFrom(address from, uint256 amount) external;
}