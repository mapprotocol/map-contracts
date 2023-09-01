// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "./IKlaytn.sol";

interface IVerifyTool {

    function bytesToAddressArray(bytes memory data)
    external
    pure
    returns (address[] memory);

    function decodeVote(bytes memory _votes)
    external
    pure
    returns(IKlaytn.Vote memory votes);


    function checkReceiptsConcat(bytes[] memory _receipts, bytes32 _receiptsHash)
    external
    pure
    returns (bool);

    function getBlockNewHash(IKlaytn.BlockHeader memory header)
    external
    pure
    returns (bytes32 blockHash, bytes32 removeSealHash, IKlaytn.ExtraData memory ext);


    function checkHeaderParam(IKlaytn.BlockHeader memory header)
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
