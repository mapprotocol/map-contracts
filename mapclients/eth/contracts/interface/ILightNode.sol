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

    struct txProve{
        bytes keyIndex;
        bytes[] prove;
        bytes expectedValue;
    }

    struct txLogs{
        bytes PostStateOrStatus;
        uint CumulativeGasUsed;
        bytes Bloom;
        Log[] logs;
    }

    struct Log {
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

    struct proveData {
        blockHeader header;
        txLogs logs;
        txProve prove;
    }

    function verifyProofData(proveData memory _proveData, G2 memory aggPk) external view returns (bool success, string memory message);

    //
    function updateBlockHeader(blockHeader memory bh, G2 memory aggPk) external;

    //G1
    function initialize(uint256 _threshold, address[] memory validaters,G1[] memory _pairKeys, uint256[] memory _weights,uint epoch, uint epochSize) external;
}