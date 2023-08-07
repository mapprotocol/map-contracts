// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "./RLPEncode.sol";
import "./RLPReader.sol";
import "./ProofLib.sol";

library Types {
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    struct BlockHeader {
        bytes32 parentHash;
        uint256 height;
        uint256 timestamp;
        address author;
        bytes32 transactionsRoot;
        bytes32 deferredStateRoot;
        bytes32 deferredReceiptsRoot;
        bytes32 deferredLogsBloomHash;
        uint256 blame;
        uint256 difficulty;
        bool adaptive;
        uint256 gasLimit;
        bytes32[] refereeHashes;
        bytes[] custom;
        uint256 nonce;
        bytes32 posReference;
    }

    function encodeBlockHeader(BlockHeader memory header) internal pure returns (bytes memory) {
        uint256 len = header.posReference == bytes32(0) ? 14 : 15;
        len += header.custom.length;
        bytes[] memory list = new bytes[](len);

        list[0] = RLPEncode.encodeBytes(abi.encodePacked(header.parentHash));
        list[1] = RLPEncode.encodeUint(header.height);
        list[2] = RLPEncode.encodeUint(header.timestamp);
        list[3] = RLPEncode.encodeAddress(header.author);
        list[4] = RLPEncode.encodeBytes(abi.encodePacked(header.transactionsRoot));
        list[5] = RLPEncode.encodeBytes(abi.encodePacked(header.deferredStateRoot));
        list[6] = RLPEncode.encodeBytes(abi.encodePacked(header.deferredReceiptsRoot));
        list[7] = RLPEncode.encodeBytes(abi.encodePacked(header.deferredLogsBloomHash));
        list[8] = RLPEncode.encodeUint(header.blame);
        list[9] = RLPEncode.encodeUint(header.difficulty);
        list[10] = RLPEncode.encodeUint(header.adaptive ? 1 : 0);
        list[11] = RLPEncode.encodeUint(header.gasLimit);
        list[12] = _encodeBytes32Array(header.refereeHashes);
        list[13] = RLPEncode.encodeUint(header.nonce);

        uint256 offset = 14;

        if (header.posReference != bytes32(0)) {
            list[offset] = _encodePosReference(header.posReference);
            offset++;
        }

        for (uint256 i = 0; i < header.custom.length; i++) {
            // add as raw data
            list[offset + i] = header.custom[i];
        }

        return RLPEncode.encodeList(list);
    }

    function _encodePosReference(bytes32 pos) private pure returns (bytes memory) {
        if (pos == bytes32(0)) {
            bytes[] memory list = new bytes[](0);
            return RLPEncode.encodeList(list);
        } else {
            bytes[] memory list = new bytes[](1);
            list[0] = RLPEncode.encodeBytes(abi.encodePacked(pos));
            return RLPEncode.encodeList(list);
        }
    }

    function _encodeBytes32Array(bytes32[] memory data) private pure returns (bytes memory) {
        bytes[] memory list = new bytes[](data.length);

        for (uint256 i = 0; i < data.length; i++) {
            list[i] = RLPEncode.encodeBytes(abi.encodePacked(data[i]));
        }

        return RLPEncode.encodeList(list);
    }

    function computeBlockHash(BlockHeader memory header) internal pure returns (bytes32) {
        bytes memory encoded = encodeBlockHeader(header);
        return keccak256(encoded);
    }

    struct BlockHeaderWrapper {
        bytes32 parentHash;
        uint256 height;
        bytes32 deferredReceiptsRoot;
    }

    function rlpDecodeBlockHeader(bytes memory header) internal pure returns (BlockHeaderWrapper memory wrapper) {
        RLPReader.Iterator memory iter = RLPReader.toRlpItem(header).iterator();
        wrapper.parentHash = bytes32(iter.next().toUintStrict());
        wrapper.height = iter.next().toUint();
        iter.next(); // timestamp
        iter.next(); // miner
        iter.next(); // txs root
        iter.next(); // state root
        wrapper.deferredReceiptsRoot = bytes32(iter.next().toUintStrict());
    }

    struct ReceiptProof {
        // Continuous block headers (RLP encoded), that head is for receipts root,
        // and tail block should be relayed on chain.
        bytes[] headers;

        bytes blockIndex;
        ProofLib.ProofNode[] blockProof;

        bytes32 receiptsRoot;
        bytes index;
        bytes receipt; // RLP encoded
        ProofLib.ProofNode[] receiptProof;
    }

    struct TxReceipt {
        uint256 accumulatedGasUsed;
        uint256 gasFee;
        bool gasSponsorPaid;
        bytes logBloom;
        TxLog[] logs;
        uint8 outcomeStatus;
        bool storageSponsorPaid;
        StorageChange[] storageCollateralized;
        StorageChange[] storageReleased;
    }

    struct TxLog {
        address addr;
        bytes32[] topics;
        bytes data;
        uint8 space; // Native: 1, Ethereum: 2
    }

    struct StorageChange {
        address account;
        uint64 collaterals;
    }

    function encodeReceipt(TxReceipt memory receipt) internal pure returns (bytes memory) {
        bytes[] memory list = new bytes[](9);

        list[0] = RLPEncode.encodeUint(receipt.accumulatedGasUsed);
        list[1] = RLPEncode.encodeUint(receipt.gasFee);
        list[2] = RLPEncode.encodeBool(receipt.gasSponsorPaid);
        list[3] = RLPEncode.encodeBytes(receipt.logBloom);
        list[4] = encodeLogs(receipt.logs);
        list[5] = RLPEncode.encodeUint(receipt.outcomeStatus);
        list[6] = RLPEncode.encodeBool(receipt.storageSponsorPaid);
        list[7] = _encodeStorageChanges(receipt.storageCollateralized);
        list[8] = _encodeStorageChanges(receipt.storageReleased);

        return RLPEncode.encodeList(list);
    }

    function encodeLogs(TxLog[] memory logs) internal pure returns (bytes memory) {
        bytes[] memory list = new bytes[](logs.length);

        for (uint256 i = 0; i < logs.length; i++) {
            require(logs[i].space == 1 || logs[i].space == 2, "Types: invalid space of receipt");

            bytes[] memory tmp = new bytes[](logs[i].space == 1 ? 3 : 4);

            tmp[0] = RLPEncode.encodeAddress(logs[i].addr);
            tmp[1] = _encodeBytes32Array(logs[i].topics);
            tmp[2] = RLPEncode.encodeBytes(logs[i].data);

            // append space for eSpace
            if (logs[i].space == 2) {
                tmp[3] = RLPEncode.encodeUint(2);
            }

            list[i] = RLPEncode.encodeList(tmp);
        }

        return RLPEncode.encodeList(list);
    }

    function _encodeStorageChanges(StorageChange[] memory changes) private pure returns (bytes memory) {
        bytes[] memory list = new bytes[](changes.length);
        bytes[] memory tmp = new bytes[](2);

        for (uint256 i = 0; i < changes.length; i++) {
            tmp[0] = RLPEncode.encodeAddress(changes[i].account);
            tmp[1] = RLPEncode.encodeUint(changes[i].collaterals);
            list[i] = RLPEncode.encodeList(tmp);
        }

        return RLPEncode.encodeList(list);
    }

}
