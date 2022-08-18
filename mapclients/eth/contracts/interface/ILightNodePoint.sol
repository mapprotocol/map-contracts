// SPDX-License-Identifier: MIT



pragma solidity ^0.8.0;

import "./IBLSPoint.sol";

interface ILightNodePoint is IBLSPoint {
    //Map chain block header
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
        //extraData: Expand the information field to store information suchas committee member changes and voting.
        bytes extraData;
        bytes mixDigest;
        bytes nonce;
        uint256 baseFee;
    }


    struct txReceipt {
        uint256 receiptType;
        bytes postStateOrStatus;
        uint256 cumulativeGasUsed;
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

    //Committee change information corresponds to extraData in blockheader
    struct istanbulExtra {
        //Addresses of added committee members
        address[] validators;
        //The public key of the added committee member
        bytes[] addedPubKey;
        //G1 public key of the added committee member
        bytes[] addedG1PubKey;
        //Members removed from the previous committee are removed by bit 1 after binary encoding
        uint256 removeList;
        //The signature of the previous committee on the current header
        //Reference for specific signature and encoding rules
        //https://docs.maplabs.io/develop/map-relay-chain/consensus/epoch-and-block/aggregatedseal#calculate-the-hash-of-the-block-header
        bytes seal;
        //Information on current committees
        istanbulAggregatedSeal aggregatedSeal;
        //Information on the previous committee
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
