// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./ILightNode.sol";

interface IVerifyToolV2 {
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
        bytes extraData; // Expand the field to store information such as committee member changes and voting
        bytes mixDigest;
        bytes nonce;
        uint256 baseFee;
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

    // verify mpt proof and return the receipt logs
    function verifyTrieProof(
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure returns (bool success, bytes memory logs);

    // verify mpt proof and return the _logIndex log
    function verifyTrieProofWithLog(
        uint256 _logIndex,
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure returns (bool success, ILightNode.txLog memory log);


    function unsafeDecodeTxReceipt(bytes memory _receiptRlp) external pure returns (bytes memory logHash);

    function checkHeader(
        uint256 _blockNumber,
        bytes memory _header,
        bytes memory _signHeader,
        IVerifyToolV2.istanbulExtra memory ist,
        bool checkValidator,
        bool getReceiptRoot
    ) external pure returns (bool success, string memory message, address coinbase, bytes32 receiptRoot);

    function verifyHeaderHash(
        address _coinbase,
        bytes memory _seal,
        bytes32 headerBytesHash
    ) external pure returns (bool ret);
}
