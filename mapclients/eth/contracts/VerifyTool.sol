// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./lib/RLPReader.sol";
import "./lib/RLPEncode.sol";
import "./interface/ILightNodePoint.sol";
import "./lib/MPT.sol";

contract VerifyTool is ILightNodePoint {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;
    using MPT for MPT.MerkleProof;


    function getVerifyTrieProof(receiptProof memory _receiptProof)
    public
    pure
    returns (bool success, string memory message){
        bytes memory expectedValue = getVerifyExpectedValueHash(_receiptProof.receipt);
        MPT.MerkleProof memory mp;
        mp.expectedRoot = bytes32(_receiptProof.header.receiptHash);
        mp.key = _receiptProof.keyIndex;
        mp.proof = _receiptProof.proof;
        mp.keyIndex = 0;
        mp.proofIndex = 0;
        mp.expectedValue = expectedValue;
        success = MPT.verifyTrieProof(mp);
        if (!success) {
            message = "receipt mismatch";
        } else {
            message = "success";
        }
    }

    function decodeHeader(bytes memory rlpBytes)
    public
    pure
    returns (blockHeader memory bh){
        RLPReader.RLPItem[] memory ls = rlpBytes.toRlpItem().toList();
        bh = blockHeader({
        parentHash : ls[0].toBytes(),
        coinbase : ls[1].toAddress(),
        root : ls[2].toBytes(),
        txHash : ls[3].toBytes(),
        receiptHash : ls[4].toBytes(),
        number : ls[6].toUint(),
        extraData : ls[10].toBytes(),
        bloom : ls[5].toBytes(),
        gasLimit : ls[7].toUint(),
        gasUsed : ls[8].toUint(),
        time : ls[9].toUint(),
        mixDigest : ls[11].toBytes(),
        nonce : ls[12].toBytes(),
        baseFee : ls[13].toUint()
        });
    }


    function encodeHeader(blockHeader memory bh)
    public
    pure
    returns (bytes memory output){
        bytes[] memory list = new bytes[](14);
        list[0] = RLPEncode.encodeBytes(bh.parentHash);
        list[1] = RLPEncode.encodeAddress(bh.coinbase);
        list[2] = RLPEncode.encodeBytes(bh.root);
        list[3] = RLPEncode.encodeBytes(bh.txHash);
        list[4] = RLPEncode.encodeBytes(bh.receiptHash);
        list[5] = RLPEncode.encodeBytes(bh.bloom);
        list[6] = RLPEncode.encodeUint(bh.number);
        list[7] = RLPEncode.encodeUint(bh.gasLimit);
        list[8] = RLPEncode.encodeUint(bh.gasUsed);
        list[9] = RLPEncode.encodeUint(bh.time);
        list[10] = RLPEncode.encodeBytes(bh.extraData);
        list[11] = RLPEncode.encodeBytes(bh.mixDigest);
        list[12] = RLPEncode.encodeBytes(bh.nonce);
        list[13] = RLPEncode.encodeUint(bh.baseFee);
        output = RLPEncode.encodeList(list);
    }

    function decodeExtraData(bytes memory extraData)
    public
    pure
    returns (istanbulExtra memory ist){
        bytes memory decodeBytes = splitExtra(extraData);
        RLPReader.RLPItem[] memory ls = decodeBytes.toRlpItem().toList();
        RLPReader.RLPItem[] memory item0 = ls[0].toList();
        RLPReader.RLPItem[] memory item1 = ls[1].toList();
        RLPReader.RLPItem[] memory item2 = ls[2].toList();
        RLPReader.RLPItem memory item3 = ls[3];
        RLPReader.RLPItem memory item4 = ls[4];
        RLPReader.RLPItem memory item5 = ls[5];
        RLPReader.RLPItem memory item6 = ls[6];
        address[] memory validatorTemp = new address[](item0.length);
        bytes[] memory addedPubKeyTemp = new bytes[](item1.length);
        bytes[] memory addedG1PubKeyTemp = new bytes[](item2.length);
        if (item0.length > 0) {
            for (uint256 i = 0; i < item0.length; i++) {
                validatorTemp[i] = item0[i].toAddress();
            }
        }
        if (item1.length > 0) {
            for (uint256 j = 0; j < item1.length; j++) {
                addedPubKeyTemp[j] = item1[j].toBytes();
            }
        }
        if (item2.length > 0) {
            for (uint256 k = 0; k < item2.length; k++) {
                addedG1PubKeyTemp[k] = item2[k].toBytes();
            }
        }
        ist = istanbulExtra({
        validators : validatorTemp,
        addedPubKey : addedPubKeyTemp,
        addedG1PubKey : addedG1PubKeyTemp,
        removeList : item3.toUint(),
        seal : item4.toBytes(),
        aggregatedSeal : istanbulAggregatedSeal({
        bitmap : item5.toList()[0].toUint(),
        signature : item5.toList()[1].toBytes(),
        round : item5.toList()[2].toUint()
        }),
        parentAggregatedSeal : istanbulAggregatedSeal({
        bitmap : item6.toList()[0].toUint(),
        signature : item6.toList()[1].toBytes(),
        round : item6.toList()[2].toUint()
        })
        });

    }

    function encodeTxLog(txLog[] memory _txLogs)
    public
    pure
    returns (bytes memory output){
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

    function decodeTxLog(bytes memory logsHash)
    public
    pure
    returns (txLog[] memory _txLogs){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
         _txLogs = new txLog[](ls.length);
        for(uint256 i = 0; i< ls.length; i++){
            bytes[] memory topic = new bytes[](ls[i].toList()[1].toList().length);
            for(uint256 j = 0; j < ls[i].toList()[1].toList().length; j++){
                topic[j] = ls[i].toList()[1].toList()[j].toBytes();
            }
            _txLogs[i] = txLog({
            addr:ls[i].toList()[0].toAddress(),
            topics : topic,
            data : ls[i].toList()[2].toBytes()
            });
        }
    }

    function getBlcokHash(blockHeader memory bh)
    public
    pure
    returns (bytes32){
        istanbulExtra memory ist = decodeExtraData(bh.extraData);
        bytes memory extraDataPre = splitExtra32(bh.extraData);
        bh.extraData = deleteAgg(ist, extraDataPre);
        bytes memory headerWithoutAgg = encodeHeader(bh);
        return keccak256(abi.encodePacked(headerWithoutAgg));
    }

    function verifyHeader(bytes memory rlpHeader)
    public
    pure
    returns (bool ret, bytes32 headerHash){
        blockHeader memory bh = decodeHeader(rlpHeader);
        istanbulExtra memory ist = decodeExtraData(bh.extraData);
        headerHash = getHeaderHash(bh);
        ret = verifySign(
            ist.seal,
            headerHash,
            bh.coinbase
        );
    }



    function splitExtra(bytes memory extra)
    internal
    pure
    returns (bytes memory newExtra){
        newExtra = new bytes(extra.length - 32);
        uint256 n = 0;
        for (uint256 i = 32; i < extra.length; i++) {
            newExtra[n] = extra[i];
            n = n + 1;
        }
        return newExtra;
    }

    function splitExtra32(bytes memory extra)
    internal
    pure
    returns (bytes memory newExtra){
        newExtra = new bytes(32);
        uint m = 0;
        for (uint i = 0; i < 32; i++) {
            newExtra[m] = extra[i];
            m = m + 1;
        }
        return newExtra;
    }


    function getVerifyExpectedValueHash(txReceipt memory _txReceipt)
    internal
    pure
    returns (bytes memory output){
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

    function getHeaderHash(blockHeader memory bh)
    public
    pure
    returns (bytes32){
        istanbulExtra memory ist = decodeExtraData(bh.extraData);
        bytes memory extraDataPre = splitExtra32(bh.extraData);
        bh.extraData = deleteAgg(ist, extraDataPre);
        bh.extraData = deleteSealAndAgg(ist, bh.extraData);
        bytes memory headerWithoutSealAndAgg = encodeHeader(bh);
        bytes32 hash2 = keccak256(abi.encodePacked(headerWithoutSealAndAgg));
        return keccak256(abi.encodePacked(hash2));
    }


    function deleteAgg(istanbulExtra memory ist, bytes memory extraDataPre)
    internal
    pure
    returns (bytes memory newExtra){
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
        bytes[] memory list = new bytes[](7);
        list[0] = RLPEncode.encodeList(list1);
        list[1] = RLPEncode.encodeList(list2);
        list[2] = RLPEncode.encodeList(list3);
        list[3] = RLPEncode.encodeUint(ist.removeList);
        list[4] = RLPEncode.encodeBytes(ist.seal);
        list[5] = new bytes(4);
        list[5][0] = bytes1(uint8(195));
        list[5][1] = bytes1(uint8(128));
        list[5][2] = bytes1(uint8(128));
        list[5][3] = bytes1(uint8(128));
        list[6] = encodeAggregatedSeal(ist.parentAggregatedSeal.bitmap, ist.parentAggregatedSeal.signature, ist.parentAggregatedSeal.round);
        bytes memory b = RLPEncode.encodeList(list);
        bytes memory output = new bytes(b.length + 32);
        for (uint i = 0; i < b.length + 32; i++) {
            if (i < 32) {
                output[i] = extraDataPre[i];
            } else {
                output[i] = b[i - 32];
            }
        }
        newExtra = output;
    }


    function deleteSealAndAgg(istanbulExtra memory ist, bytes memory rlpHeader)
    internal
    pure
    returns (bytes memory newExtra){
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
        bytes[] memory list = new bytes[](7);
        list[0] = RLPEncode.encodeList(list1);
        list[1] = RLPEncode.encodeList(list2);
        list[2] = RLPEncode.encodeList(list3);
        list[3] = RLPEncode.encodeUint(ist.removeList);
        list[4] = new bytes(1);
        list[4][0] = bytes1(uint8(128));
        list[5] = new bytes(4);
        list[5][0] = bytes1(uint8(195));
        list[5][1] = bytes1(uint8(128));
        list[5][2] = bytes1(uint8(128));
        list[5][3] = bytes1(uint8(128));
        list[6] = encodeAggregatedSeal(
            ist.parentAggregatedSeal.bitmap,
            ist.parentAggregatedSeal.signature,
            ist.parentAggregatedSeal.round
        );
        bytes memory b = RLPEncode.encodeList(list);
        newExtra = abi.encodePacked(bytes32(rlpHeader), b);
    }


    function encodeAggregatedSeal(uint256 bitmap, bytes memory signature, uint256 round)
    internal
    pure
    returns (bytes memory output) {
        bytes memory output1 = RLPEncode.encodeUint(bitmap);
        bytes memory output2 = RLPEncode.encodeBytes(signature);
        bytes memory output3 = RLPEncode.encodeUint(round);
        bytes[] memory list = new bytes[](3);
        list[0] = output1;
        list[1] = output2;
        list[2] = output3;
        output = RLPEncode.encodeList(list);
    }

    function verifySign(bytes memory seal, bytes32 hash, address coinbase)
    internal
    pure
    returns (bool) {
        (bytes32 r, bytes32 s, uint8 v) = splitSignature(seal);
        v = v + 27;
        return coinbase == ecrecover(hash, v, r, s);
    }

    function splitSignature(bytes memory sig)
    internal
    pure
    returns
    (bytes32 r, bytes32 s, uint8 v){
        require(sig.length == 65, "invalid signature length");
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
    }

}
