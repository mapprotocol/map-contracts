// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;
import "./ILightNodePoint.sol";

interface IVerifyTool is ILightNodePoint {

    function bytesToAddressArray(bytes memory data)
    external
    pure
    returns (address[] memory);

    function decodeVote(bytes memory _votes)
    external
    pure
    returns(Vote memory votes);

    function decodeHeaderExtraData(bytes memory _extBytes)
    external
    pure
    returns (bytes memory extTop,ExtraData memory extData);

    function checkReceiptsConcat(bytes[] memory _receipts, bytes32 _receiptsHash)
    external
    pure
    returns (bool);

    function checkReceiptsOriginal(ReceiptProofOriginal memory _proof)
    external
    view
    returns (bool success,bytes memory logs);

    function getBlockNewHash(BlockHeader memory header, bytes memory extraData,bytes memory _removeSealExtra)
    external
    pure
    returns (bytes32 headerBytes,bytes32 removeSealHeaderBytes);

    function getRemoveSealExtraData(ExtraData memory _ext, bytes memory _extHead, bool _keepSeal)
    external
    pure
    returns (bytes memory, bytes memory);

    function checkHeaderParam(BlockHeader memory header)
    external
    view
    returns (bool);

    function recoverSigner(bytes memory seal, bytes32 hash)
    external
    pure
    returns (address);

    function isRepeat(address[] memory _miners, address _miner, uint256 _limit)
    external
    pure
    returns (bool);
}
