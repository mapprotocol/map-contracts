// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {
    function verifyProofData(bytes memory _receiptProof) external view returns (bool success, string memory message, bytes memory logs);

    function updateBlockHeader(bytes memory _blockHeader) external;

    function updateLightClient(bytes memory _data) external;

    function clientState() external view returns(bytes memory);

    function headerHeight() external view returns (uint256 height);

    function verifiableHeaderRange() external view returns (uint256, uint256);
}