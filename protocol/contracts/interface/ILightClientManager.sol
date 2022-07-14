// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightClientManager {
    function updateBlockHeader(uint256 _chainId, bytes memory _blackHeader) external;
    function register(uint256 _chainId, address _contract) external;
}