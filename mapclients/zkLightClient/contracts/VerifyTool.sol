// SPDX-License-Identifier: MIT

pragma solidity 0.8.21;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/RLPEncode.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./interface/ILightNodePoint.sol";

contract VerifyTool is ILightNodePoint {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint8 constant STRING_SHORT_START = 0x80;
    uint8 constant STRING_SHORT_ARRAY_START = 0xc3;

    function getVerifyTrieProof(
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure returns (bool success, string memory message) {
        bytes memory expectedValue = getVerifyExpectedValueHash(_receiptType, _receiptRlp);
        success = MPT.verify(expectedValue, _keyIndex, _proof, _receiptHash);
        if (!success) {
            message = "mpt verification failed";
        } else {
            message = "success";
        }
    }

    function decodeHeader(bytes memory rlpBytes) external pure returns (blockHeader memory bh) {
        RLPReader.RLPItem[] memory ls = rlpBytes.toRlpItem().toList();
        bh = blockHeader({
            parentHash: ls[0].toBytes(),
            coinbase: ls[1].toAddress(),
            root: ls[2].toBytes(),
            txHash: ls[3].toBytes(),
            receiptHash: ls[4].toBytes(),
            number: ls[6].toUint(),
            extraData: ls[10].toBytes(),
            bloom: ls[5].toBytes(),
            gasLimit: ls[7].toUint(),
            gasUsed: ls[8].toUint(),
            time: ls[9].toUint(),
            mixDigest: ls[11].toBytes(),
            nonce: ls[12].toBytes(),
            baseFee: ls[13].toUint()
        });
    }

    function encodeHeader(
        blockHeader memory _bh,
        bytes memory _deleteAggBytes,
        bytes memory _deleteSealAndAggBytes
    ) external pure returns (bytes memory deleteAggHeaderBytes, bytes memory deleteSealAndAggHeaderBytes) {
        bytes[] memory list = new bytes[](14);
        list[0] = RLPEncode.encodeBytes(_bh.parentHash);
        list[1] = RLPEncode.encodeAddress(_bh.coinbase);
        list[2] = RLPEncode.encodeBytes(_bh.root);
        list[3] = RLPEncode.encodeBytes(_bh.txHash);
        list[4] = RLPEncode.encodeBytes(_bh.receiptHash);
        list[5] = RLPEncode.encodeBytes(_bh.bloom);
        list[6] = RLPEncode.encodeUint(_bh.number);
        list[7] = RLPEncode.encodeUint(_bh.gasLimit);
        list[8] = RLPEncode.encodeUint(_bh.gasUsed);
        list[9] = RLPEncode.encodeUint(_bh.time);
        list[10] = RLPEncode.encodeBytes(_deleteAggBytes);
        list[11] = RLPEncode.encodeBytes(_bh.mixDigest);
        list[12] = RLPEncode.encodeBytes(_bh.nonce);
        list[13] = RLPEncode.encodeUint(_bh.baseFee);
        deleteAggHeaderBytes = RLPEncode.encodeList(list);
        list[10] = RLPEncode.encodeBytes(_deleteSealAndAggBytes);
        deleteSealAndAggHeaderBytes = RLPEncode.encodeList(list);
    }

    function manageAgg(
        istanbulExtra memory ist
    ) external pure returns (bytes memory deleteAggBytes, bytes memory deleteSealAndAggBytes) {
        bytes[] memory list1 = new bytes[](ist.validators.length);
        bytes[] memory list2 = new bytes[](ist.addedPubKey.length);
        bytes[] memory list3 = new bytes[](ist.addedG1PubKey.length);
        for (uint256 i = 0; i < ist.validators.length; i++) {
            list1[i] = RLPEncode.encodeAddress(ist.validators[i]);
        }
        for (uint256 i = 0; i < ist.addedPubKey.length; i++) {
            list2[i] = RLPEncode.encodeBytes(ist.addedPubKey[i]);
        }
        for (uint256 i = 0; i < ist.addedG1PubKey.length; i++) {
            list3[i] = RLPEncode.encodeBytes(ist.addedG1PubKey[i]);
        }
        bytes[] memory manageList = new bytes[](7);
        manageList[0] = RLPEncode.encodeList(list1);
        manageList[1] = RLPEncode.encodeList(list2);
        manageList[2] = RLPEncode.encodeList(list3);
        manageList[3] = RLPEncode.encodeUint(ist.removeList);
        manageList[4] = RLPEncode.encodeBytes(ist.seal);
        manageList[5] = new bytes(4);
        manageList[5][0] = bytes1(STRING_SHORT_ARRAY_START);
        manageList[5][1] = bytes1(STRING_SHORT_START);
        manageList[5][2] = bytes1(STRING_SHORT_START);
        manageList[5][3] = bytes1(STRING_SHORT_START);
        manageList[6] = encodeAggregatedSeal(
            ist.parentAggregatedSeal.bitmap,
            ist.parentAggregatedSeal.signature,
            ist.parentAggregatedSeal.round
        );
        deleteAggBytes = RLPEncode.encodeList(manageList);

        manageList[4] = new bytes(1);
        manageList[4][0] = bytes1(STRING_SHORT_START);

        deleteSealAndAggBytes = RLPEncode.encodeList(manageList);
    }

    function encodeTxLog(txLog[] memory _txLogs) external pure returns (bytes memory output) {
        bytes[] memory listLog = new bytes[](_txLogs.length);
        bytes[] memory loglist = new bytes[](3);
        for (uint256 j = 0; j < _txLogs.length; j++) {
            loglist[0] = RLPEncode.encodeAddress(_txLogs[j].addr);
            bytes[] memory loglist1 = new bytes[](_txLogs[j].topics.length);
            for (uint256 i = 0; i < _txLogs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes(_txLogs[j].topics[i]);
            }
            loglist[1] = RLPEncode.encodeList(loglist1);
            loglist[2] = RLPEncode.encodeBytes(_txLogs[j].data);
            bytes memory logBytes = RLPEncode.encodeList(loglist);
            listLog[j] = logBytes;
        }
        output = RLPEncode.encodeList(listLog);
    }

    function decodeTxLog(bytes memory logsHash) external pure returns (txLog[] memory _txLogs) {
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            RLPReader.RLPItem[] memory item = ls[i].toList();
            RLPReader.RLPItem[] memory firstItemList = item[1].toList();
            bytes[] memory topic = new bytes[](firstItemList.length);
            for (uint256 j = 0; j < firstItemList.length; j++) {
                topic[j] = firstItemList[j].toBytes();
            }
            _txLogs[i] = txLog({addr: item[0].toAddress(), topics: topic, data: item[2].toBytes()});
        }
    }

    function decodeTxReceipt(bytes memory _receiptRlp) external pure returns (bytes memory logHash) {
        RLPReader.RLPItem[] memory ls = _receiptRlp.toRlpItem().toList();
        logHash = RLPReader.toRlpBytes(ls[3]);
    }

    function verifyHeader(
        address _coinbase,
        bytes memory _seal,
        bytes memory _headerWithoutSealAndAgg
    ) public pure returns (bool ret, bytes32 headerHash) {
        headerHash = keccak256(abi.encodePacked(keccak256(abi.encodePacked(_headerWithoutSealAndAgg))));
        ret = verifySign(_seal, headerHash, _coinbase);
    }

    function getVerifyExpectedValueHash(
        uint256 _receiptType,
        bytes memory receiptRlp
    ) internal pure returns (bytes memory expectedValue) {
        if (_receiptType == 0) {
            return receiptRlp;
        } else {
            bytes memory tempType = abi.encode(_receiptType);
            bytes1 tip = tempType[31];
            return abi.encodePacked(tip, receiptRlp);
        }
    }

    function splitExtra(bytes memory extra) internal pure returns (bytes memory newExtra) {
        require(extra.length > 32, "Invalid extra result type");
        newExtra = new bytes(extra.length - 32);
        uint256 n = 0;
        for (uint256 i = 32; i < extra.length; i++) {
            newExtra[n] = extra[i];
            n = n + 1;
        }
        return newExtra;
    }

    function encodeAggregatedSeal(
        uint256 bitmap,
        bytes memory signature,
        uint256 round
    ) internal pure returns (bytes memory output) {
        bytes memory output1 = RLPEncode.encodeUint(bitmap);
        bytes memory output2 = RLPEncode.encodeBytes(signature);
        bytes memory output3 = RLPEncode.encodeUint(round);
        bytes[] memory list = new bytes[](3);
        list[0] = output1;
        list[1] = output2;
        list[2] = output3;
        output = RLPEncode.encodeList(list);
    }

    function verifySign(bytes memory seal, bytes32 hash, address coinbase) internal pure returns (bool) {
        (bytes32 r, bytes32 s, uint8 v) = splitSignature(seal);
        if (v <= 1) {
            v = v + 27;
        }
        return coinbase == ECDSA.recover(hash, v, r, s);
    }

    function splitSignature(bytes memory sig) internal pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "invalid signature length");
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
    }
}
