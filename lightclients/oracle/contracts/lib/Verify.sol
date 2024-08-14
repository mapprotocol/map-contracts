// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";

// import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";

library Verify {
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;
    using RLPReader for bytes;

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
        bytes memory bytesReceipt = _receipt.txReceipt;
        bytes memory expectedValue = bytesReceipt;
        if (_receipt.receiptType > 0) {
            expectedValue = abi.encodePacked(bytes1(uint8(_receipt.receiptType)), expectedValue);
        }
        success = MPT.verify(expectedValue, _receipt.keyIndex, _receipt.proof, _receiptsRoot);
        if (success) logs = bytesReceipt.toRlpItem().safeGetItemByIndex(3).toRlpBytes(); // list length must be 4
    }
}
