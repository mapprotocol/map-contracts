// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

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
        bytes extraData;
        bytes32 mixHash;
        bytes nonce;
        uint256 baseFeePerGas;
        bytes32 withdrawalsRoot;
        uint256 blobGasUsed;
        uint256 excessBlobGas;
        bytes32 parentBeaconBlockRoot;
    }

    struct ReceiptProof {
        TxReceipt txReceipt;
        bytes keyIndex;
        bytes[] proof;
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

    struct VoteData  {
        uint256  SourceNumber;  // The source block number should be the latest justified block number.
        bytes32  SourceHash;    // The block hash of the source block.
        uint256  TargetNumber;  // The target block number which validator wants to vote for.
        bytes32  TargetHash;    // The block hash of the target block.
    }
    struct VoteAttestation  {
        uint64    VoteAddressSet; // The bitset marks the voted validators.
        bytes     Signature;     // The aggregated BLS signature of the voted validators' signatures.
        VoteData  Data;          // The vote data for fast finality.   
    }

    struct UpdateHeader {
       BlockHeader[] headers;
       VoteAttestation[2]  voteAttestations;
    }

    struct ProofData {
        UpdateHeader updateHeader;
        ReceiptProof receiptProof;
    }
