// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@mapprotocol/protocol/contracts/lib/MPT.sol";
import "@mapprotocol/protocol/contracts/lib/LogDecode.sol";

import "@mapprotocol/protocol/contracts/interface/ILightVerifier.sol";


library Verify {
    struct ReceiptProof {
        bytes txReceipt;
        uint256 receiptType;
        bytes keyIndex;
        bytes[] proof;
    }

    function _validateProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mptVerify
    ) internal pure returns (bool success, bytes memory logs) {
        success = _verifyMptProof(_receiptsRoot, _receipt);
        if (success) {
            logs = LogDecode.getLogsFromReceipt(_receipt.txReceipt);
        }
    }

    function _validateProofWithLog(
        uint256 _logIndex,
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt
    ) internal pure returns (bool success, ILightVerifier.txLog memory log) {
        success = _verifyMptProof(_receiptsRoot, _receipt);
        if (success) {
            log = LogDecode.decodeTxLogFromReceipt(_logIndex, _receipt.txReceipt);
        }
        return (success, log);
    }

    function _verifyMptProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt
    ) internal pure returns (bool success) {
        bytes memory expectedValue = _receipt.txReceipt;
        if (_receipt.receiptType > 0) {
            expectedValue = abi.encodePacked(bytes1(uint8(_receipt.receiptType)), expectedValue);
        }
        bytes32 expectedHash = keccak256(expectedValue);
        return MPT.verify(expectedHash, _receipt.keyIndex, _receipt.proof, _receiptsRoot);
    }
}
