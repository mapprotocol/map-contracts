// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {
    event UpdateBlockHeader(address indexed maintainer, uint256 indexed blockHeight);

    event ClientNotifySend(address indexed sender, uint256 indexed blockHeight, bytes notifyData);

    function updateBlockHeader(bytes memory _blockHeader) external;

    function updateLightClient(bytes memory _data) external;

    // @notice Notify light client to relay the block
    // @param _data - notify data, if no data set it to empty
    function notifyLightClient(address _from, bytes memory _data) external;

    // @notice Validate the receipt according to the block header and receipt merkel proof
    //         Using block header number and block receipt root cache to optimize the validation gas cost.
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message
    // @return logs - the logs included in the receipt
    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external returns (bool success, string memory message, bytes memory logs);

    // @notice Validate the receipt according to the block header and receipt merkel proof
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message
    // @return logs - the logs included in the receipt
    function verifyProofData(
        bytes memory _receiptProof
    ) external view returns (bool success, string memory message, bytes memory logs);

    // Get client state
    function clientState() external view returns (bytes memory);

    function finalizedState(bytes memory _data) external view returns (bytes memory);

    // @notice Get the light client block height
    // @return height - current block height or slot number
    function headerHeight() external view returns (uint256 height);

    //
    function verifiableHeaderRange() external view returns (uint256, uint256);

    // @notice Check whether the block can be verified
    // @return
    function isVerifiable(uint256 _blockHeight, bytes32 _hash) external view returns (bool);

    // @notice Get the light client type
    // @return - 1 default light client
    //           2 zk light client
    //           3 oracle client
    function nodeType() external view returns (uint256);
}
