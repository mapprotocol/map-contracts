// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightVerifier {

    struct txLog {
        address addr;
        bytes32[] topics;
        bytes data;
    }

    event ClientNotifySend(address indexed sender, uint256 indexed blockHeight, bytes notifyData);

    // @notice Notify light client to relay the block
    // @param _data - notify data, if no data set it to empty
    function notifyLightClient(address _from, bytes memory _data) external;

    // @notice Validate the receipt according to the block header and receipt merkel proof
    //         Using block header number and block receipt root cache to optimize the validation gas cost.
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message, return null if verify successfully
    // @return logs - the logs included in the receipt
    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external returns (bool success, string memory message, bytes memory logs);

    // @notice Validate the receipt according to the block header and receipt merkel proof
    //         Using block header number and block receipt root cache to optimize the validation gas cost.
    // @param _cache - whether store the receipt root as cache, will extra cost store gas
    //          but will save the header proof verification if will verify more tx receipts in one block
    // @param _logIndex - the log index from 0
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message, return null if verify successfully
    // @return log - the log corresponding to the index
    function verifyProofDataWithCache(
        bool _cache,
        uint256 _logIndex,
        bytes memory _receiptProofBytes
    ) external returns (bool success, string memory message, txLog memory log);

    // @notice Validate the receipt according to the block header and receipt merkel proof
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message, return null if verify successfully
    // @return logs - the rlp logs included in the receipt
    function verifyProofData(
        bytes memory _receiptProof
    ) external view returns (bool success, string memory message, bytes memory logs);

    // @notice Validate the receipt according to the block header and receipt merkel proof
    // @param _logIndex - the log index from 0
    // @param _receiptProof - the bytes to receipt proof
    // @return success - verification result
    // @return message - the result message, return null if verify successfully
    // @return log - the log corresponding to the index
    function verifyProofData(
        uint256 _logIndex,
        bytes memory _receiptProof
    ) external view returns (bool success, string memory message, txLog memory log);

    function verifiableHeaderRange() external view returns (uint256, uint256);

    // @notice Check whether the block can be verified
    // @param _blockHeight - the block number
    // @param _hash - the block receipt root
    // @return
    function isVerifiable(uint256 _blockHeight, bytes32 _hash) external view returns (bool);

    // @notice Get the light client type
    // @return - 1 default light client
    //           2 zk light client
    //           3 oracle client
    //           4 oracle client v2
    function nodeType() external view returns (uint256);
}
