// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "./ILightNodeMAP.sol";
import "../lib/LedgerInfoLib.sol";
import "../lib/Types.sol";

interface ILightNode is ILightNodeMAP {

    struct State {
        uint256 epoch;                  // pos block epoch
        uint256 round;                  // pos block round

        uint256 earliestBlockNumber;    // earliest pow block number relayed
        uint256 finalizedBlockNumber;   // last finalized pow block number

        uint256 blocks;                 // number of relayed pow blocks
        uint256 maxBlocks;              // maximum number of pow blocks to retain
    }

    function initialize(
        address controller,
        address ledgerInfoUtil,
        address mptVerify,
        LedgerInfoLib.EpochState memory committee,
        LedgerInfoLib.LedgerInfoWithSignatures memory ledgerInfo
    ) external;

    function relayPOS(LedgerInfoLib.LedgerInfoWithSignatures memory ledgerInfo) external;
    // relay RLP encoded pow block headers
    function relayPOW(bytes[] memory headers) external;
    function removeBlockHeader(uint256 limit) external;

    function verifyReceiptProof(Types.ReceiptProof memory proof) external view returns (bool);

    function state() external view returns(State memory);
    function nearestPivot(uint256 height) external view returns (uint256);
}
