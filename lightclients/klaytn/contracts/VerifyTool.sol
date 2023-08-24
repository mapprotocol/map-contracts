// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./lib/RLPReader.sol";
import "./lib/RLPEncode.sol";
import "./lib/MPT.sol";
import "./interface/ILightNodePoint.sol";

contract VerifyTool is ILightNodePoint {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;
    using MPT for MPT.MerkleProof;


    uint256 constant EXTRA_VANITY = 32;

    function bytesToAddressArray(bytes memory _data)
    external
    pure
    returns (address[] memory)
    {

        uint256 dataNb = _data.length / 20;
        address[] memory dataList = new address[](dataNb);
        uint256 index = 0;
        for (uint256 i = 20; i <= _data.length; i = i + 20) {
            address temp;
            assembly {
                temp := mload(add(_data, i))
            }
            dataList[index] = temp;
            index++;
        }
        return dataList;
    }

    function decodeVote(bytes memory _votes)
    external
    pure
    returns
    (Vote memory votes)
    {
        RLPReader.RLPItem[] memory ls = _votes.toRlpItem().toList();
        return ( Vote({
        validator : ls[0].toAddress(),
        key : ls[1].toBytes(),
        value : ls[2].toBytes()
        }));
    }

    function decodeHeaderExtraData(bytes memory _extBytes)
    external
    pure
    returns (bytes memory extTop, ExtraData memory extData)
    {
        (bytes memory extraHead,bytes memory istBytes) = _splitExtra(_extBytes);

        RLPReader.RLPItem[] memory ls = istBytes.toRlpItem().toList();
        RLPReader.RLPItem[] memory itemValidators = ls[0].toList();
        RLPReader.RLPItem[] memory itemCommittedSeal = ls[2].toList();

        bytes memory _seal = ls[1].toBytes();
        address[] memory _validators = new address[](itemValidators.length);
        for (uint256 i = 0; i < itemValidators.length; i++) {
            _validators[i] = itemValidators[i].toAddress();
        }
        bytes[] memory _committedSeal = new bytes[](itemCommittedSeal.length);
        for (uint256 i = 0; i < itemCommittedSeal.length; i++) {
            _committedSeal[i] = itemCommittedSeal[i].toBytes();
        }

        return (extraHead, ExtraData({
        validators : _validators,
        seal : _seal,
        committedSeal : _committedSeal
        }));
    }


    function checkReceiptsConcat(
        bytes[] memory _receipts,
        bytes32 _receiptsHash)
    external
    pure
    returns (bool)
    {
        bytes memory receiptsAll;
        for (uint i = 0; i < _receipts.length; i++) {
            receiptsAll = bytes.concat(receiptsAll, _receipts[i]);
        }
        return keccak256(receiptsAll) == _receiptsHash;
    }



    function getBlockNewHash(
        BlockHeader memory _header,
        bytes memory _extraData,
        bytes memory _removeSealExtra)
    external
    pure
    returns (bytes32 headerBytes,bytes32 removeSealHeaderBytes)
    {
        bytes[] memory list = new bytes[](15);
        list[0] = RLPEncode.encodeBytes(_header.parentHash);
        list[1] = RLPEncode.encodeAddress(_header.reward);
        list[2] = RLPEncode.encodeBytes(_header.stateRoot);
        list[3] = RLPEncode.encodeBytes(_header.transactionsRoot);
        list[4] = RLPEncode.encodeBytes(_header.receiptsRoot);
        list[5] = RLPEncode.encodeBytes(_header.logsBloom);
        list[6] = RLPEncode.encodeUint(_header.blockScore);
        list[7] = RLPEncode.encodeUint(_header.number);
        list[8] = RLPEncode.encodeUint(_header.gasUsed);
        list[9] = RLPEncode.encodeUint(_header.timestamp);
        list[10] = RLPEncode.encodeUint(_header.timestampFoS);
        list[11] = RLPEncode.encodeBytes(_extraData);
        list[12] = RLPEncode.encodeBytes(_header.governanceData);
        list[13] = RLPEncode.encodeBytes(_header.voteData);
        list[14] = RLPEncode.encodeUint(_header.baseFee);
        headerBytes = keccak256(RLPEncode.encodeList(list));
        list[11] = RLPEncode.encodeBytes(_removeSealExtra);
        removeSealHeaderBytes = keccak256(RLPEncode.encodeList(list));
    }


    function getRemoveSealExtraData(
        ExtraData memory _ext,
        bytes memory _extHead,
        bool _keepSeal)
    external
    pure
    returns (bytes memory, bytes memory)
    {
        bytes[] memory listExt = new bytes[](3);
        bytes[] memory listValidators = new bytes[](_ext.validators.length);

        for (uint i = 0; i < _ext.validators.length; i ++) {
            listValidators[i] = RLPEncode.encodeAddress(_ext.validators[i]);
        }
        listExt[0] = RLPEncode.encodeList(listValidators);
        if (!_keepSeal) {
            listExt[1] = RLPEncode.encodeBytes("");
        } else {
            listExt[1] = RLPEncode.encodeBytes(_ext.seal);
        }
        listExt[2] = RLPEncode.encodeList(new bytes[](0));

        bytes memory output = RLPEncode.encodeList(listExt);
        _extHead[31] = 0;
        return (abi.encodePacked(_extHead, output), _ext.seal);
    }



    function checkHeaderParam(BlockHeader memory _header)
    external
    view
    returns (bool)
    {
        if (_header.timestamp + 60 > block.timestamp) {return false;}
        if (_header.blockScore == 0) {return false;}
        return true;
    }

    function recoverSigner(
        bytes memory _seal,
        bytes32 _hash)
    external
    pure
    returns (address)
    {
        (bytes32 r, bytes32 s, uint8 v) = _splitSignature(_seal);
        if (v <= 1) {
            v = v + 27;
        }
        return ECDSA.recover(_hash, v, r, s);
    }


    function isRepeat(
        address[] memory _miners,
        address _miner,
        uint256 _limit)
    external
    pure
    returns (bool)
    {
        for (uint256 i = 0; i < _limit; i++) {
            if (_miners[i] == _miner) {
                return true;
            }
        }

        return false;
    }


    function checkReceiptsOriginal(ReceiptProofOriginal memory _proof)
    external
    pure
    returns (bool success,bytes memory logs)
    {

        bytes memory bytesReceipt = _encodeReceipt(_proof.txReceipt);

        MPT.MerkleProof memory mp = MPT.MerkleProof({
        expectedRoot: bytes32(_proof.header.receiptsRoot),
        key: _proof.keyIndex,
        proof: _proof.proof,
        keyIndex: 0,
        proofIndex: 0,
        expectedValue: bytesReceipt
        });

        success = MPT.verifyTrieProof(mp);
        uint256 rlpIndex = 3;
        logs = bytesReceipt.toRlpItem().toList()[rlpIndex].toRlpBytes();

        return (success,logs);
    }

    function _splitExtra(bytes memory _extra)
    internal
    pure
    returns (
        bytes memory extraHead,
        bytes memory extraEnd)
    {
        require(_extra.length >= 32, "Invalid extra result type");
        extraEnd = new bytes(_extra.length - EXTRA_VANITY);
        extraHead = new bytes(EXTRA_VANITY);
        for (uint256 i = 0; i < _extra.length; i++) {
            if (i < EXTRA_VANITY) {
                extraHead[i] = _extra[i];
            } else {
                extraEnd[i - EXTRA_VANITY] = _extra[i];
            }
        }
        return (extraHead, extraEnd);
    }

    function _splitSignature(bytes memory _sig)
    internal
    pure
    returns (bytes32 r, bytes32 s, uint8 v)
    {
        require(_sig.length == 65, "invalid signature length");
        assembly {
            r := mload(add(_sig, 32))
            s := mload(add(_sig, 64))
            v := byte(0, mload(add(_sig, 96)))
        }
    }

    function _encodeReceipt(TxReceipt memory _txReceipt)
    internal
    pure
    returns (bytes memory output)
    {
        bytes[] memory list = new bytes[](4);
        list[0] = RLPEncode.encodeBytes(_txReceipt.postStateOrStatus);
        list[1] = RLPEncode.encodeUint(_txReceipt.cumulativeGasUsed);
        list[2] = RLPEncode.encodeBytes(_txReceipt.bloom);
        bytes[] memory listLog = new bytes[](_txReceipt.logs.length);
        bytes[] memory loglist = new bytes[](3);
        for (uint256 j = 0; j < _txReceipt.logs.length; j++) {
            loglist[0] = RLPEncode.encodeAddress(_txReceipt.logs[j].addr);
            bytes[] memory loglist1 = new bytes[](
                _txReceipt.logs[j].topics.length
            );
            for (uint256 i = 0; i < _txReceipt.logs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes(
                    _txReceipt.logs[j].topics[i]
                );
            }
            loglist[1] = RLPEncode.encodeList(loglist1);
            loglist[2] = RLPEncode.encodeBytes(_txReceipt.logs[j].data);
            bytes memory logBytes = RLPEncode.encodeList(loglist);
            listLog[j] = logBytes;
        }
        list[3] = RLPEncode.encodeList(listLog);
        output = RLPEncode.encodeList(list);
    }


}
