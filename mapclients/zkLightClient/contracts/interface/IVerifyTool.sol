// SPDX-License-Identifier: MIT

pragma solidity 0.8.21;
import "./ILightNodePoint.sol";

interface IVerifyTool is ILightNodePoint {
    function getVerifyTrieProof(
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure returns (bool success, string memory message);

    function decodeHeader(bytes memory rlpBytes) external view returns (blockHeader memory bh);

    function encodeHeader(
        blockHeader memory _bh,
        bytes memory _deleteAggBytes,
        bytes memory _deleteSealAndAggBytes
    ) external view returns (bytes memory deleteAggHeaderBytes, bytes memory deleteSealAndAggHeaderBytes);

    function decodeExtraData(bytes memory extraData) external view returns (istanbulExtra memory ist);

    function manageAgg(
        istanbulExtra memory ist
    ) external pure returns (bytes memory deleteAggBytes, bytes memory deleteSealAndAggBytes);

    function encodeTxLog(txLog[] memory _txLogs) external view returns (bytes memory output);

    function decodeTxLog(bytes memory logsHash) external view returns (txLog[] memory _txLogs);

    function decodeTxReceipt(bytes memory receiptRlp) external pure returns (bytes memory logHash);

    function verifyHeader(
        address _coinbase,
        bytes memory _seal,
        bytes memory _headerWithoutSealAndAgg
    ) external view returns (bool ret, bytes32 headerHash);
}
