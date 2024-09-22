// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "./lib/LogDecode.sol";

abstract contract LogDecoder {

    function _decodeTxLogs(bytes memory logsHash) internal pure returns (ILightVerifier.txLog[] memory _txLogs) {

        return LogDecode.decodeTxLogs(logsHash);

    }

    function _decodeTxLog(bytes memory logsHash, uint256 logIndex) internal pure returns (ILightVerifier.txLog memory _txLog) {
        return LogDecode.decodeTxLog(logsHash, logIndex);
    }
}
