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

    function decodeTxLogs(bytes memory logs)
    internal
    pure
    returns (txLog[] memory _txLogs) {
        RLPReader.RLPItem[] memory ls = logs.toRlpItem().toList();
        _txLogs = new txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            RLPReader.RLPItem[] memory item = ls[i].toList();

            require(item.length >= 3, "log length to low");

            RLPReader.RLPItem[] memory firstItemList = item[1].toList();
            bytes[] memory topic = new bytes[](firstItemList.length);
            for (uint256 j = 0; j < firstItemList.length; j++) {
                topic[j] = firstItemList[j].toBytes();
            }
            _txLogs[i] = txLog({
            addr : item[0].toAddress(),
            topics : topic,
            data : item[2].toBytes()
            });
        }
    }

}
