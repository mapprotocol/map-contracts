// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./RLPReader.sol";
import "../utils/Utils.sol";

import "../interface/ILightVerifier.sol";

library LogDecode {
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    function decodeTxLogs(bytes memory logsHash) internal pure returns (ILightVerifier.txLog[] memory _txLogs) {
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new ILightVerifier.txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            _txLogs[i] = _decodeTxLog(ls[i]);
        }
    }

    function decodeTxLog(bytes memory logsHash, uint256 logIndex) internal pure returns (ILightVerifier.txLog memory _txLog) {
        RLPReader.RLPItem memory ls = logsHash.toRlpItem().safeGetItemByIndex(logIndex);
        _txLog = _decodeTxLog(ls);
    }

    function _decodeTxLog(RLPReader.RLPItem memory item) private pure returns (ILightVerifier.txLog memory _txLog) {
        RLPReader.RLPItem[] memory items = item.toList();
        require(items.length >= 3, "log length to low");
        RLPReader.RLPItem[] memory firstItemList = items[1].toList();
        bytes32[] memory topic = new bytes32[](firstItemList.length);
        for (uint256 j = 0; j < firstItemList.length; j++) {
            topic[j] = firstItemList[j].toBytes32();
        }
        _txLog = ILightVerifier.txLog({addr: items[0].toAddress(), topics: topic, data: items[2].unsafeToBytes()});
    }
}
