// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "../interface/IEvent.sol";
import "./RLPReader.sol";
import "../utils/Utils.sol";

library EvmDecoder {

    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    function decodeTxLogs(bytes memory logsHash)
    internal
    pure
    returns (IEvent.txLog[] memory _txLogs) {
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new IEvent.txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            RLPReader.RLPItem[] memory item = ls[i].toList();

            require(item.length >= 3, "log length to low");

            RLPReader.RLPItem[] memory firstItemList = item[1].toList();
            bytes[] memory topic = new bytes[](firstItemList.length);
            for (uint256 j = 0; j < firstItemList.length; j++) {
                topic[j] = firstItemList[j].toBytes();
            }
            _txLogs[i] = IEvent.txLog({
            addr : item[0].toAddress(),
            topics : topic,
            data : item[2].toBytes()
            });
        }
    }

}
