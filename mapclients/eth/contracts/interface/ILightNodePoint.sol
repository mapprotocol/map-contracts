// SPDX-License-Identifier: MIT



pragma solidity ^0.8.0;

import "./IBLSPoint.sol";

interface ILightNodePoint is IBLSPoint {
    struct blockHeader {
        bytes parentHash;
        address coinbase;
        bytes root;
        bytes txHash;
        bytes receiptHash;
        bytes bloom;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 time;
        bytes extraData;
        bytes mixDigest;
        bytes nonce;
        uint256 baseFee;
    }


    struct txReceipt{
        uint256  receiptType;
        bytes   postStateOrStatus;
        uint256   cumulativeGasUsed;
        bytes bloom;
        txLog[] logs;
    }

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    struct istanbulAggregatedSeal {
        uint256 bitmap;
        bytes signature;
        uint256 round;

    }

    struct istanbulExtra {
        address[] validators;
        bytes[] addedPubKey;
        bytes[] addedG1PubKey;
        uint256 removeList;
        bytes seal;
        istanbulAggregatedSeal aggregatedSeal;
        istanbulAggregatedSeal parentAggregatedSeal;
    }

    struct receiptProof {
        blockHeader header;
        G2 aggPk;
        txReceipt receipt;
        bytes keyIndex;
        bytes[] proof;
    }

}
