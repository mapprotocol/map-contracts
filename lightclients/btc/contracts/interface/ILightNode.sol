// SPDX-License-Identifier: MIT

pragma solidity ^0.8.7;

interface ILightNode {

    event UpdateBlockHeader(address indexed account, uint256 indexed blockHeight);

    function updateBlockHeader(bytes memory header) external;

    function verifiableHeaderRange() external view returns (uint256, uint256);

    function headerHeight() external view returns (uint256);

    function verifyProofData(bytes memory _proofData) external view returns (bool);
}
