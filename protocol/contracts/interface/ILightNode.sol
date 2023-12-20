// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {

    event UpdateBlockHeader(address indexed maintainer, uint256 indexed blockHeight);


    function updateBlockHeader(bytes memory _blockHeader) external;

    function updateLightClient(bytes memory _data) external;

    // @notice Validate the receipt according to the block header and receipt merkel proof
    //         Using block header number and block receipt root cache to optimize the validation gas cost.
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message
    // @return logs - the logs included in the receipt
    function verifyProofDataWithCache(bytes memory _receiptProof) external
    returns (bool success, string memory message,bytes memory logs);


    // @notice Validate the receipt according to the block header and receipt merkel proof
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message
    // @return logs - the logs included in the receipt
    function verifyProofData(bytes memory _receiptProof) external view returns (bool success, string memory message, bytes memory logs);


    // Get client state
    function clientState() external view returns(bytes memory);

    function finalizedState(bytes memory _data) external view returns(bytes memory);

    // @notice Get the light client block height
    // @return height - current block height or slot number
    function headerHeight() external view returns (uint256 height);

    //
    function verifiableHeaderRange() external view returns (uint256, uint256);
}