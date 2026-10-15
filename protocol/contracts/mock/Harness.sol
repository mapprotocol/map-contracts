// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "../LogDecoder.sol";
import "../lib/LogDecode.sol";
import "../utils/Utils.sol";
import "../MaintainerManager.sol";
import {RLPReader} from "../lib/RLPReader.sol";

/// @dev Exposes the internal library functions so they can be tested directly.
contract LogDecodeHarness is LogDecoder {
    function decodeTxLogs(bytes memory logs) external pure returns (ILightVerifier.txLog[] memory) {
        return _decodeTxLogs(logs);
    }

    function decodeTxLog(bytes memory logs, uint256 idx) external pure returns (ILightVerifier.txLog memory) {
        return _decodeTxLog(logs, idx);
    }

    function decodeLogRlp(bytes memory log) external pure returns (ILightVerifier.txLog memory) {
        return LogDecode.decodeLogRlp(log);
    }

    function getLogsFromReceipt(bytes memory receipt) external pure returns (bytes memory) {
        return LogDecode.getLogsFromReceipt(receipt);
    }

    function getLogsFromTypedReceipt(uint256 t, bytes memory receipt) external pure returns (bytes memory) {
        return LogDecode.getLogsFromTypedReceipt(t, receipt);
    }

    function decodeTxLogFromReceipt(uint256 idx, bytes memory receipt) external pure returns (ILightVerifier.txLog memory) {
        return LogDecode.decodeTxLogFromReceipt(idx, receipt);
    }

    function decodeTxLogFromTypedReceipt(
        uint256 idx,
        uint256 t,
        bytes memory receipt
    ) external pure returns (ILightVerifier.txLog memory) {
        return LogDecode.decodeTxLogFromTypedReceipt(idx, t, receipt);
    }
}

contract UtilsHarness {
    function checkBytes(bytes memory a, bytes memory b) external pure returns (bool) {
        return Utils.checkBytes(a, b);
    }

    function fromBytes(bytes memory b) external pure returns (address) {
        return Utils.fromBytes(b);
    }

    function toBytes(address a) external pure returns (bytes memory) {
        return Utils.toBytes(a);
    }

    function splitExtra(bytes memory b) external pure returns (bytes memory) {
        return Utils.splitExtra(b);
    }

    function hexStrToBytes(bytes memory b) external pure returns (bytes memory) {
        return Utils.hexStrToBytes(b);
    }

    function isValidNearAddress(bytes memory b) external pure returns (bool) {
        return Utils.isValidNearAddress(b);
    }

    function isValidAddress(bytes memory b, uint256 t) external pure returns (bool) {
        return Utils.isValidAddress(b, t);
    }
}

/// @dev Same storage layout as MaintainerManager, used to test UUPS upgrades.
contract MaintainerManagerV2 is MaintainerManager {
    function version() external pure returns (uint256) {
        return 2;
    }
}

/// @dev Whitelisted staker without a payable fallback, so any payout to it fails.
contract NoReceiveStaker {
    function deposit(address mm) external payable {
        MaintainerManager(payable(mm)).deposit{value: msg.value}();
    }

    function withdraw(address mm, uint256 amt) external {
        MaintainerManager(payable(mm)).withdraw(amt);
    }
}

/// @dev Staker that tries to re-enter withdraw() from its receive hook.
contract ReentrantStaker {
    MaintainerManager public mm;
    bool public reentered;

    constructor(address _mm) {
        mm = MaintainerManager(payable(_mm));
    }

    function deposit() external payable {
        mm.deposit{value: msg.value}();
    }

    function withdraw(uint256 amt) external {
        mm.withdraw(amt);
    }

    receive() external payable {
        reentered = true;
        mm.withdraw(0);
    }
}

/// @dev Raw RLPReader usage pattern taken from downstream light clients.
contract RLPReaderHarness {
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    function firstItem(bytes memory b) external pure returns (bytes memory) {
        return b.toRlpItem().toList()[0].toBytes();
    }

    // raw single-item accessors
    function toUint(bytes memory b) external pure returns (uint256) {
        return b.toRlpItem().toUint();
    }

    function toUintStrict(bytes memory b) external pure returns (uint256) {
        return b.toRlpItem().toUintStrict();
    }

    function toBytes32(bytes memory b) external pure returns (bytes32) {
        return b.toRlpItem().toBytes32();
    }

    function toBytes(bytes memory b) external pure returns (bytes memory) {
        return b.toRlpItem().toBytes();
    }

    function toBoolean(bytes memory b) external pure returns (bool) {
        return b.toRlpItem().toBoolean();
    }

    function toAddress(bytes memory b) external pure returns (address) {
        return b.toRlpItem().toAddress();
    }

    function listLength(bytes memory b) external pure returns (uint256) {
        return b.toRlpItem().toList().length;
    }

    function isList(bytes memory b) external pure returns (bool) {
        return b.toRlpItem().isList();
    }

    function itemByIndex(bytes memory b, uint256 i) external pure returns (bytes memory) {
        return b.toRlpItem().safeGetItemByIndex(i).toRlpBytes();
    }
}
