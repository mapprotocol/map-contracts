// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;
import "./ILightNodePoint.sol";

interface IVerifyTool is ILightNodePoint {
    function getVerifyTrieProof(receiptProof memory _receiptProof)
    external
    pure
    returns (bool success, string memory message);

    function decodeHeader(bytes memory rlpBytes)
    external
    view
    returns (blockHeader memory bh);

    function encodeHeader(blockHeader memory bh)
    external
    view
    returns (bytes memory output);

    function decodeExtraData(bytes memory extraData)
    external
    view
    returns (istanbulExtra memory ist);

    function encodeTxLog(txLog[] memory _txLogs)
    external
    view
    returns (bytes memory output);

    function decodeTxLog(bytes memory logsHash)
    external
    view
    returns (txLog[] memory _txLogs);

    function getBlockHash(blockHeader memory bh)
    external
    view
    returns (bytes32);

    function verifyHeader(bytes memory rlpHeader)
    external
    view
    returns (bool ret, bytes32 headerHash);
}
