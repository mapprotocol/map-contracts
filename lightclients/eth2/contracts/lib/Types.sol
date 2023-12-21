// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

library Types {
    struct BeaconBlockHeader {
        uint64 slot;
        uint64 proposerIndex;
        bytes32 parentRoot;
        bytes32 stateRoot;
        bytes32 bodyRoot;
    }

    struct BlockHeader {
        bytes32 parentHash;
        bytes32 sha3Uncles;
        address miner;
        bytes32 stateRoot;
        bytes32 transactionsRoot;
        bytes32 receiptsRoot;
        bytes logsBloom;
        uint256 difficulty;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 timestamp;
        bytes extraData; // 96
        bytes32 mixHash;
        bytes nonce; // 8
        uint256 baseFeePerGas;
        bytes32 withdrawalsRoot;
    }

    struct TxReceipt {
        uint256 receiptType;
        bytes postStateOrStatus;
        uint256 cumulativeGasUsed;
        bytes bloom;
        TxLog[] logs;
    }

    struct TxLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    struct ReceiptProof {
        BlockHeader header;
        TxReceipt txReceipt;
        bytes keyIndex;
        bytes[] proof;
    }

    struct SyncCommittee {
        bytes pubkeys; // 48 * 512
        bytes aggregatePubkey; // 48
    }

    struct SyncAggregate {
        bytes syncCommitteeBits;
        bytes syncCommitteeSignature;
    }

    struct Execution {
        bytes32 parentHash;
        address feeRecipient;
        bytes32 stateRoot;
        bytes32 receiptsRoot;
        bytes logsBloom;
        bytes32 prevRandao;
        uint256 blockNumber;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 timestamp;
        bytes extraData; // 96
        uint256 baseFeePerGas;
        bytes32 blockHash;
        bytes32 transactionsRoot;
        bytes32 withdrawalsRoot;
    }

    struct LightClientUpdate {
        BeaconBlockHeader attestedHeader;
        SyncCommittee nextSyncCommittee;
        bytes32[] nextSyncCommitteeBranch;
        BeaconBlockHeader finalizedHeader;
        bytes32[] finalityBranch;
        Execution finalizedExecution;
        bytes32[] executionBranch;
        SyncAggregate syncAggregate;
        uint64 signatureSlot;
    }
}
