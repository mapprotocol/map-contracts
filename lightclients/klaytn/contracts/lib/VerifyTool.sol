// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/RLPEncode.sol";
import "../interface/IVerifyTool.sol";

contract VerifyTool is IVerifyTool {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint256 constant EXTRA_VANITY = 32;   // Fixed number of extra-data bytes reserved for validator vanity

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
    (IKlaytn.Vote memory votes)
    {
        RLPReader.RLPItem[] memory ls = _votes.toRlpItem().toList();
        return ( IKlaytn.Vote({
        validator : ls[0].toAddress(),
        key : ls[1].toBytes(),
        value : ls[2].toBytes()
        }));
    }


    function decodeHeaderExtraData(bytes memory _extBytes)
    public
    pure
    returns (IKlaytn.ExtraData memory extData, bytes memory extWithoutCommitteeSeal, bytes memory extWithoutCommitteeSealAndSeal)
    {
        (bytes memory extraHead, bytes memory istBytes) = _splitExtra(_extBytes);

        RLPReader.RLPItem[] memory ls = istBytes.toRlpItem().toList();
        RLPReader.RLPItem[] memory itemValidators = ls[0].toList();
        bytes memory _seal = ls[1].toBytes();
        RLPReader.RLPItem[] memory itemCommittedSeal = ls[2].toList();

        address[] memory _validators = new address[](itemValidators.length);
        for (uint256 i = 0; i < itemValidators.length; i++) {
            _validators[i] = itemValidators[i].toAddress();
        }
        bytes[] memory _committedSeal = new bytes[](itemCommittedSeal.length);
        for (uint256 i = 0; i < itemCommittedSeal.length; i++) {
            _committedSeal[i] = itemCommittedSeal[i].toBytes();
        }

        extData = IKlaytn.ExtraData({
            validators : _validators,
            seal : _seal,
            committedSeal : _committedSeal
        });

        bytes[] memory listExt = new bytes[](3);
        listExt[0] = ls[0].toRlpBytes();
        listExt[1] = ls[1].toRlpBytes();
        listExt[2] = RLPEncode.encodeList(new bytes[](0));

        bytes memory output = RLPEncode.encodeList(listExt);

        extraHead[EXTRA_VANITY - 1] = 0;   // set round

        extWithoutCommitteeSeal = abi.encodePacked(extraHead, output);

        listExt[1] = RLPEncode.encodeBytes("");
        output = RLPEncode.encodeList(listExt);

        extWithoutCommitteeSealAndSeal = abi.encodePacked(extraHead, output);
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



    function getBlockHashAndExtData(IKlaytn.BlockHeader memory _header)
    external
    pure
    returns (bytes32 blockHash, bytes32 removeSealHash, IKlaytn.ExtraData memory ext)
    {
        bytes memory extWithoutCommitteeSeal;
        bytes memory extWithoutCommitteeSealAndSeal;

        (ext, extWithoutCommitteeSeal, extWithoutCommitteeSealAndSeal) = decodeHeaderExtraData(_header.extraData);

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
        list[11] = RLPEncode.encodeBytes(extWithoutCommitteeSeal);
        list[12] = RLPEncode.encodeBytes(_header.governanceData);
        list[13] = RLPEncode.encodeBytes(_header.voteData);
        list[14] = RLPEncode.encodeUint(_header.baseFee);
        blockHash = keccak256(RLPEncode.encodeList(list));

        list[11] = RLPEncode.encodeBytes(extWithoutCommitteeSealAndSeal);
        removeSealHash = keccak256(RLPEncode.encodeList(list));
    }


    function checkHeaderParam(IKlaytn.BlockHeader memory _header)
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


    function _splitExtra(bytes memory _extra)
    internal
    pure
    returns (
        bytes memory extraHead,
        bytes memory extraEnd)
    {
        require(_extra.length >= EXTRA_VANITY, "Invalid extra result type");
        extraEnd = new bytes(_extra.length - EXTRA_VANITY);
        extraHead = new bytes(EXTRA_VANITY);

        // TODO: optimize
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

}
