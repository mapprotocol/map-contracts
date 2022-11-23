// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface ILightNode{
    function verifyProofData(bytes memory _receiptProof) external returns (bool success, string memory message, bytes memory logs);

    function updateBlockHeader(bytes memory _blackHeader) external;

    function headerHeight() external view returns (uint256);

    function currentNumberAndHash(uint256 number) external view returns(uint256 ,bytes memory);
}