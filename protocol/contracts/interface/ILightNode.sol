// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {

    event UpdateBlockHeader(address indexed maintainer, uint256 indexed blockHeight);


    function updateBlockHeader(bytes memory _blockHeader) external;

    function updateLightClient(bytes memory _data) external;


    // Verify the validity of the transaction according to the header, receipt
    // The interface will be updated later to return logs
    function verifyProofData(bytes memory _receiptProof) external view returns (bool success, string memory message, bytes memory logs);


    // Get client state
    function clientState() external view returns(bytes memory);

    function finalizedState(bytes memory _data) external view returns(bytes memory);

    function headerHeight() external view returns (uint256 height);

    //
    function verifiableHeaderRange() external view returns (uint256, uint256);
}