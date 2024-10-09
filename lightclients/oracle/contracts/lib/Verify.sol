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
        address 
    ) internal pure returns (bool success, bytes memory logs) {
        success = _verifyMptProof(_receiptsRoot, _receipt);
        if (success) logs = LogDecode.getLogsFromTypedReceipt(_receipt.receiptType, _receipt.txReceipt);
    }

    function _validateProofWithLog(
        uint256 _logIndex,
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt
    ) internal pure returns (bool success, ILightVerifier.txLog memory log) {
        success = _verifyMptProof(_receiptsRoot, _receipt);
        if (success) log = LogDecode.decodeTxLogFromTypedReceipt(_logIndex, _receipt.receiptType, _receipt.txReceipt);
    }

    function _verifyMptProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt
    ) internal pure returns (bool success) {
        bytes32 expectedHash = keccak256(_receipt.txReceipt);
        return MPT.verify(expectedHash, _receipt.keyIndex, _receipt.proof, _receiptsRoot);
    }

    //addr(20) + 4 + 4(topic num) +4(data len) + topic[] + data
    function _decodeLogFromBytes(bytes memory _logBytes) internal pure returns(ILightVerifier.txLog memory log){
        require(_logBytes.length >= 32, "invalid logBytes length");
        address addr;
        uint256 topicNum;
        uint256 dataLen;
        uint256 point;
        assembly {
            //skip 32 byte data length
            point := add(_logBytes, 32)
            let firstWord := mload(point)
            addr := shr(96, firstWord)
            topicNum := shr(32, and(firstWord, 0x000000000000000000000000000000000000000000000000ffffffff00000000))
            dataLen := and(firstWord, 0x00000000000000000000000000000000000000000000000000000000ffffffff)
        }
        log.addr = addr;
        log.topics = new bytes32[](topicNum);

        for(uint256 i = 0; i < topicNum; i++) {
            point += 32;
            bytes32 t;
            assembly {
               t := mload(point)
            }
            log.topics[i] = t;
        }

        bytes memory d;
        assembly {
            mstore(point, dataLen)
            d := point
        }
        log.data = d;
    }
}
