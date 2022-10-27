// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {
    struct BeaconBlockHeader {
        uint64 slot;
        uint64 proposerIndex;
        bytes32 parentRoot;
        bytes32 stateRoot;
        bytes32 bodyRoot;
    }

    struct BlockHeader {
        bytes parentHash;
        bytes sha3Uncles;
        address miner;
        bytes stateRoot;
        bytes transactionsRoot;
        bytes receiptsRoot;
        bytes logsBloom;
        uint256 difficulty;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 timestamp;
        bytes extraData;  // 96
        bytes mixHash;
        bytes nonce;     // 8
        uint256 baseFeePerGas;
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
        bytes pubkeys;  // 48 * 512
        bytes aggregatePubkey; // 48
    }

    struct SyncAggregate {
        bytes syncCommitteeBits;
        bytes syncCommitteeSignature;
    }

    struct LightClientUpdate {
        BeaconBlockHeader attestedHeader;
        SyncCommittee nextSyncCommittee;
        bytes32[] nextSyncCommitteeBranch;
        BeaconBlockHeader finalizedHeader;
        bytes32[] finalityBranch;
        BlockHeader finalizedExeHeader;
        bytes32[] exeFinalityBranch;
        SyncAggregate syncAggregate;
        uint64 signatureSlot;
    }

    struct LightClientState {
        BeaconBlockHeader finalizedHeader;
        SyncCommittee currentSyncCommittee;
        SyncCommittee nextSyncCommittee;
        uint64 chainID;
    }

    struct LightClientVerify {
        LightClientUpdate update;
        LightClientState state;
    }

    struct ExeHeaderUpdateInfo {
        uint256 startNumber;
        uint256 endNumber;
        bytes32 endHash;
    }

    event UpdateLightClient(address indexed account, uint256 indexed height);

    //Verify the validity of the transaction according to the header, receipt
    //The interface will be updated later to return logs
    function verifyProofData(bytes memory _receiptProof)
    external
    view
    returns (
        bool success,
        string memory message,
        bytes memory logsHash
    );

    function updateLightClient(LightClientUpdate memory lightClientUpdate) external;

    function finalizedSlot() external view returns (uint256);
}
