// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./RLPReader.sol";
import "../utils/Utils.sol";

library LogDecoder {
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    function decodeTxLog(bytes memory logs, uint256 logIndex) internal pure returns (txLog memory _txLog) {
        RLPReader.RLPItem[] memory ls = logs.toRlpItem().toList();
        require(ls.length > logIndex, "logIndex out bond");
        _txLog = _decodeTxLog(ls[logIndex]);
    }

    function decodeTxLogs(bytes memory logs) internal pure returns (txLog[] memory _txLogs) {
        RLPReader.RLPItem[] memory ls = logs.toRlpItem().toList();
        _txLogs = new txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            _txLogs[i] = _decodeTxLog(ls[i]);
        }
    }

    function _decodeTxLog(RLPReader.RLPItem memory item) private pure returns (txLog memory _txLog) {
        RLPReader.RLPItem[] memory items = item.toList();
        require(items.length >= 3, "log length too low");
        RLPReader.RLPItem[] memory firstItemList = items[1].toList();
        bytes[] memory topic = new bytes[](firstItemList.length);
        for (uint256 j = 0; j < firstItemList.length; j++) {
            topic[j] = firstItemList[j].toBytes();
        }
        _txLog = txLog({addr: items[0].toAddress(), topics: topic, data: items[2].toBytes()});
    }
}
