// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface ILightClientManager {
    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external;
    function register(uint256 _chainId, address _contract,address _blockContract) external;
    function verifyProofData(uint _chainId, bytes memory _receiptProof) external view returns (bool success, string memory message,bytes memory logs);
    function headerHeight(uint256 _chainId) external view returns (uint256);

    function verifiableHeaderRange(uint256 _chainId) external view returns (uint256, uint256);
}