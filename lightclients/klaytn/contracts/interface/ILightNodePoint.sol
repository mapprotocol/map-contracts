// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

interface ILightNodePoint {

    enum DeriveShaOriginal{
        DeriveShaOriginal,
        DeriveShaSimple,
        DeriveShaConcat
    }

    //Klaytn chain block header
    struct BlockHeader {
        bytes parentHash;
        address reward;
        bytes stateRoot;
        bytes transactionsRoot;
        bytes receiptsRoot;
        bytes logsBloom;
        uint256 blockScore;
        uint256 number;
        uint256 gasUsed;
        uint256 timestamp;
        uint256 timestampFoS;
        bytes extraData;
        //json
        bytes governanceData;
        bytes voteData;
        uint256 baseFee;
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

    struct Vote {
        address validator;
        bytes key;
        address value;
    }


    struct ExtraData {
        address[] validators;
        bytes seal;
        bytes[] committedSeal;
    }

    struct ReceiptProof {
        bytes proof;
        DeriveShaOriginal deriveSha;
    }

    struct ReceiptProofOriginal {
        BlockHeader header;
        bytes[] proof;
        TxReceipt txReceipt;
        bytes keyIndex;
    }

    struct ReceiptProofConcat{
        BlockHeader header;
        bytes[] receipts;
        uint logIndex;
    }
}
