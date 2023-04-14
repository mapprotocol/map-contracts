// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightClientManager {
    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external;

    function updateLightClient(uint256 _chainId, bytes memory _data) external;

    function verifyProofData(uint256 _chainId, bytes memory _receiptProof) external
    view returns (bool success, string memory message,bytes memory logs);

    function clientState(uint256 _chainId) external view returns(bytes memory);

    function headerHeight(uint256 _chainId) external view returns (uint256);

    function verifiableHeaderRange(uint256 _chainId) external view returns (uint256, uint256);
}
