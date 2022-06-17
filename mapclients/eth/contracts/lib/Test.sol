// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./RLPReader.sol";
import "./RLPEncode.sol";

import "hardhat/console.sol";


contract Test{
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;
    struct G1 {
        uint x;
        uint y;
    }
    struct G2 {
        uint xr;
        uint xi;
        uint yr;
        uint yi;
    }

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


    function getVerifyExpectedValueHash(txReceipt memory _txReceipt) public pure returns (bytes memory output){
        bytes[] memory list = new bytes[](4);
        list[0] = RLPEncode.encodeBytes(_txReceipt.postStateOrStatus);
        list[1] = RLPEncode.encodeUint(_txReceipt.cumulativeGasUsed);
        list[2] = RLPEncode.encodeBytes(_txReceipt.bloom);
        bytes[] memory listLog = new bytes[](_txReceipt.logs.length);
        bytes[] memory loglist = new bytes[](3);

        for (uint256 j = 0; j < _txReceipt.logs.length; j++) {
            loglist[0] = RLPEncode.encodeAddress(_txReceipt.logs[j].addr);
            bytes[] memory loglist1 = new bytes[](_txReceipt.logs[j].topics.length);
            for (uint256 i = 0; i < _txReceipt.logs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes(_txReceipt.logs[j].topics[i]);
            }
            loglist[1] = RLPEncode.encodeList(loglist1);
            loglist[2] = RLPEncode.encodeBytes(_txReceipt.logs[j].data);
            bytes memory logBytes = RLPEncode.encodeList(loglist);
            listLog[j] = logBytes;
        }
        list[3] = RLPEncode.encodeList(listLog);
        bytes memory tempType = abi.encode(_txReceipt.receiptType);
        bytes1 tip = tempType[31];
        bytes memory temp = RLPEncode.encodeList(list);
        output = abi.encodePacked(tip, temp);
    }

    function getVerifyTrieProof(receiptProof memory _receiptProof) public returns (
        bool success, string memory message){
        bytes memory expectedValue = getVerifyExpectedValueHash(_receiptProof.receipt);
        MerkleProof memory mp;
        mp.expectedRoot = bytes32(_receiptProof.header.receiptHash);
        mp.key = _receiptProof.keyIndex;
        mp.proof = _receiptProof.proof;
        mp.keyIndex = 0;
        mp.proofIndex = 0;
        mp.expectedValue = expectedValue;
        success = verifyTrieProof(mp);
        if (!success) {
            message = "receipt mismatch";
        } else {
            message = "success";
        }
    }

    struct MerkleProof {
        bytes32 expectedRoot;
        bytes key;
        bytes[] proof;
        uint256 keyIndex;
        uint256 proofIndex;
        bytes expectedValue;
    }

    function verifyTrieProof(
        MerkleProof memory data
    )  public returns (bool)
    {
        //        console.log("console.log");
        bytes memory node = data.proof[data.proofIndex];
        RLPReader.Iterator memory dec = RLPReader.toRlpItem(node).iterator();

        if (data.keyIndex == 0) {
            require(keccak256(node) == data.expectedRoot, "verifyTrieProof root node hash invalid");
        }
        else if (node.length < 32) {
            bytes32 root = bytes32(dec.next().toUint());
            require(root == data.expectedRoot, "verifyTrieProof < 32");
        }
        else {
            require(keccak256(node) == data.expectedRoot, "verifyTrieProof else");
        }

        uint256 numberItems = RLPReader.numItems(dec.item);

        // branch
        if (numberItems == 17) {
            return verifyTrieProofBranch(data);
        }
        // leaf / extension
        else if (numberItems == 2) {
            return verifyTrieProofLeafOrExtension(dec, data);
        }

        if (data.expectedValue.length == 0) return true;
        else return false;
    }

    function verifyTrieProofBranch(
        MerkleProof memory data
    )  public returns (bool)
    {
        bytes memory node = data.proof[data.proofIndex];

        if (data.keyIndex >= data.key.length) {
            bytes memory item = RLPReader.toRlpItem(node).toList()[16].toBytes();
            if (keccak256(item) == keccak256(data.expectedValue)) {
                return true;
            }
        }
        else {
            uint256 index = uint256(uint8(data.key[data.keyIndex]));
            bytes memory _newExpectedRoot = RLPReader.toRlpItem(node).toList()[index].toBytes();

            if (!(_newExpectedRoot.length == 0)) {
                data.expectedRoot = b2b32(_newExpectedRoot);
                data.keyIndex += 1;
                data.proofIndex += 1;
                return verifyTrieProof(data);
            }
        }

        if (data.expectedValue.length == 0) return true;
        else return false;
    }

    function verifyTrieProofLeafOrExtension(
        RLPReader.Iterator memory dec,
        MerkleProof memory data
    )  public returns (bool)
    {
        bytes memory nodekey = dec.next().toBytes();
        console.logBytes(nodekey);
        bytes memory nodevalue = dec.next().toBytes();
        uint256 prefix;
        assembly {
            let first := shr(248, mload(add(nodekey, 32)))
            prefix := shr(4, first)
        }

        if (prefix == 2) {
            // leaf even
            uint256 length = nodekey.length - 1;
            console.logUint(length);
            bytes memory actualKey = sliceTransform(nodekey, 1, length, false);
            bytes memory restKey = sliceTransform(data.key, 0 + data.keyIndex, length, false);
            console.logBytes(restKey);
            console.logBytes(actualKey);
            if (keccak256(data.expectedValue) == keccak256(nodevalue)) {
                if (keccak256(actualKey) == keccak256(restKey)) return true;
                if (keccak256(expandKeyEven(actualKey)) == keccak256(restKey)) return true;
            }
        }
        else if (prefix == 3) {
            // leaf odd
            bytes memory actualKey = sliceTransform(nodekey, 32, nodekey.length, true);
            bytes memory restKey = sliceTransform(data.key, 32 + data.keyIndex, data.key.length - data.keyIndex, false);
            if (keccak256(data.expectedValue) == keccak256(nodevalue)) {
                if (keccak256(actualKey) == keccak256(restKey)) return true;
                if (keccak256(expandKeyOdd(actualKey)) == keccak256(restKey)) return true;
            }
        }
        else if (prefix == 0) {
            // extension even
            uint256 extensionLength = nodekey.length - 1;
            bytes memory shared_nibbles = sliceTransform(nodekey, 33, extensionLength, false);
            bytes memory restKey = sliceTransform(data.key, 32 + data.keyIndex, extensionLength, false);
            if (
                keccak256(shared_nibbles) == keccak256(restKey) ||
                keccak256(expandKeyEven(shared_nibbles)) == keccak256(restKey)

            ) {
                data.expectedRoot = b2b32(nodevalue);
                data.keyIndex += extensionLength;
                data.proofIndex += 1;
                return verifyTrieProof(data);
            }
        }
        else if (prefix == 1) {
            // extension odd
            uint256 extensionLength = nodekey.length;
            bytes memory shared_nibbles = sliceTransform(nodekey, 32, extensionLength, true);
            bytes memory restKey = sliceTransform(data.key, 32 + data.keyIndex, extensionLength, false);
            if (
                keccak256(shared_nibbles) == keccak256(restKey) ||
                keccak256(expandKeyEven(shared_nibbles)) == keccak256(restKey)
            ) {
                data.expectedRoot = b2b32(nodevalue);
                data.keyIndex += extensionLength;
                data.proofIndex += 1;
                return verifyTrieProof(data);
            }
        }
        else {
            revert("Invalid proof");
        }
        if (data.expectedValue.length == 0) return true;
        else return false;
    }

    function b2b32(bytes memory data) pure internal returns(bytes32 part) {
        assembly {
            part := mload(add(data, 32))
        }
    }

    function sliceTransform(
        bytes memory data,
        uint256 start,
        uint256 length,
        bool removeFirstNibble
    )
     public returns(bytes memory)
    {
        uint256 slots = length / 32;
        uint256 rest = (length % 32) * 8;
        uint256 pos = 32;
        uint256 si = 0;
        uint256 source;
        bytes memory newdata = new bytes(length);
        console.logUint(111111);
        console.logBytes(data);
        console.logUint(rest);
        uint mloadValue=0;
        assembly {
            source := add(start, data)

            if removeFirstNibble {
                mstore(
                add(newdata, pos),
                shr(4, shl(4, mload(add(source, pos))))
                )
                si := 1
                pos := add(pos, 32)
            }

            mloadValue:=mload(add(data,32))

            for {let i := si} lt(i, slots) {i := add(i, 1)} {
                mstore(add(newdata, pos), mload(add(source, pos)))
                pos := add(pos, 32)
            }

            mstore(add(newdata, pos), shl(
            rest,
            shr(rest, mload(add(source, pos)))
            ))
        }
        console.logUint(mloadValue);
        return newdata;
    }

    function getNibbles(bytes1 b) internal pure returns (bytes1 nibble1, bytes1 nibble2) {
        assembly {
            nibble1 := shr(4, b)
            nibble2 := shr(4, shl(4, b))
        }
    }

    function expandKeyEven(bytes memory data) internal pure returns (bytes memory) {
        uint256 length = data.length * 2;
        bytes memory expanded = new bytes(length);

        for (uint256 i = 0 ; i < data.length; i++) {
            (bytes1 nibble1, bytes1 nibble2) = getNibbles(data[i]);
            expanded[i * 2] = nibble1;
            expanded[i * 2 + 1] = nibble2;
        }
        return expanded;
    }

    function expandKeyOdd(bytes memory data) internal pure returns(bytes memory) {
        uint256 length = data.length * 2 - 1;
        bytes memory expanded = new bytes(length);
        expanded[0] = data[0];

        for (uint256 i = 1 ; i < data.length; i++) {
            (bytes1 nibble1, bytes1 nibble2) = getNibbles(data[i]);
            expanded[i * 2 - 1] = nibble1;
            expanded[i * 2] = nibble2;
        }
        return expanded;
    }
}