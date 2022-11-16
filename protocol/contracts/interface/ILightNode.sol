// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface ILightNode {
    function verifyProofData(bytes memory _receiptProof) external view returns (bool success, string memory message, bytes memory logs);

    function updateBlockHeader(bytes memory _blockHeader) external;

    function headerHeight() external view returns (uint256 height);

    function currentNumberAndHash(uint256 number) external view returns(uint256 ,bytes memory);

    function verifiableHeaderRange() external view returns (uint256, uint256);
}