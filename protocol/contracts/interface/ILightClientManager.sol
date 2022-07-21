// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightClientManager {
    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external;
    function register(uint256 _chainId, address _contract) external;
    function verifyProofData(uint _chainId, bytes memory _receiptProof) external view returns (bool success, bytes memory logs);
}