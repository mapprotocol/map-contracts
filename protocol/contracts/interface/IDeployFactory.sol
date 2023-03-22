// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IDeployFactory {
    /**/
    function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external;

    function getAddress(bytes32 salt) external view returns (address);
}