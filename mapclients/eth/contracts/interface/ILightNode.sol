// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./IBLSPoint.sol";

interface ILightNode is IBLSPoint {
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


    //Verify the validity of the transaction according to the header, receipt, and aggPk
    //The interface will be updated later to return logs
    function verifyProofData(receiptProof memory _receiptProof) external returns (bool success, string memory message);

    //Validate headers and update validation members
    function updateBlockHeader(blockHeader memory bh, G2 memory aggPk) external;

    //Initialize the first validator
    function initialize(uint256 _threshold, address[] memory validaters, G1[] memory _pairKeys, uint256[] memory _weights,uint epoch, uint epochSize) external;
}