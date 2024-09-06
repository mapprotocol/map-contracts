// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/RLPEncode.sol";
import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";
import {
    BlockHeader,
    ReceiptProof,
    VoteData,
    VoteAttestation,
    UpdateHeader
} from "./Types.sol";

import { BLS, Bytes } from "./bls/BLS.sol";

library Helper { 
    using Bytes for bytes;
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;


    uint256 internal constant ADDRESS_LENGTH = 20;

    uint256 internal constant EXTRA_VANITY = 32;

    uint256 internal constant EPOCH_NUM = 200;

    uint256 internal constant EXTRASEAL = 65;

    uint256 internal constant BLS_PUBLICKEY_LENGTH = 48;

    bytes32 internal constant nilParentBeaconBlockRoot = 0x0000000000000000000000000000000000000000000000000000000000000001;


    function _validateProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mptVerify
    ) internal pure returns (bool success, bytes memory logs) {

        bytes memory bytesReceipt = _receipt.txReceipt;
        bytes memory expectedValue = bytesReceipt;
        if (_receipt.receiptType > 0) {
            expectedValue = abi.encodePacked(bytes1(uint8(_receipt.receiptType)), expectedValue);
        }

        success = IMPTVerify(_mptVerify).verifyTrieProof(
            _receiptsRoot,
            _receipt.keyIndex,
            _receipt.proof,
            expectedValue
        );

        if (success) logs = bytesReceipt.toRlpItem().toList()[3].toRlpBytes(); // list length must be 4
    }


    function _checkUpdateHeader(UpdateHeader memory _updateHeader) internal pure {
        bytes32 hash;
        BlockHeader[] memory headers = _updateHeader.headers;
        uint256 len = headers.length;
        for (uint i = 0; i < len; i++) {
            if(i != 0){
                require(headers[i].parentHash == hash,"invalid parent hash");
            }
            hash = _getBlockHash(headers[i]);
        }
        VoteAttestation[2] memory votes = _updateHeader.voteAttestations;
        require(
            votes[0].Data.SourceNumber < votes[0].Data.TargetNumber &&
            votes[1].Data.SourceNumber < votes[1].Data.TargetNumber &&
            votes[0].Data.TargetNumber == votes[1].Data.SourceNumber,
            "invalid VoteAttestation"
            );
        require(hash == votes[0].Data.SourceHash || hash == votes[0].Data.TargetHash,"invalid headers");
    }

    function _verifyVoteAttestation(VoteAttestation memory _vote, bytes[] memory _BLSPublicKeys) internal view {
        uint256 len = _BLSPublicKeys.length;
        require(len != 0, "empty BLSPublicKeys");
        uint64 voteAddressSet = _vote.VoteAddressSet;

        bytes[] memory uncompressPublicKeys = _vote.uncompressPublicKeys;
        uint256 count = uncompressPublicKeys.length;
        uint256 total = _BLSPublicKeys.length;
        uint256 threshold = total * 2 / 3;
        require(count >= threshold,"not enough voted");
        uint64 mask = 1;
        uint256 index;
        for (uint256 i = 0; i < total; i++) {
             
            if((voteAddressSet & mask) != 0){
                bytes memory compress = BLS.g1_compress(uncompressPublicKeys[index]);
                require(compress.equals(_BLSPublicKeys[i]),"invalid uncompressPublicKey");
                index ++;
            }
            mask = mask << 1;
        }
        require(count == index, "uncompressPublicKeys number does not match");
        bytes32 voteDataRlpHash = _getVoteDataRlpHash(_vote.Data);
        
        _fastAggregateVerify(uncompressPublicKeys, _vote.Signature, voteDataRlpHash);
    }


    function _fastAggregateVerify(bytes[] memory _BLSPublicKeys, bytes memory _signature, bytes32 _voteDataRlpHash) internal view {
        BLS.fast_aggregate_verify(_BLSPublicKeys, _voteDataRlpHash, _signature);
    }


    function _getVoteDataRlpHash(VoteData memory _data) internal pure returns(bytes32) {
        bytes[] memory _list  = new bytes[](4);
        _list[0] = RLPEncode.encodeUint(_data.SourceNumber);
        _list[1] = RLPEncode.encodeBytes(_toBytes(_data.SourceHash));
        _list[2] = RLPEncode.encodeUint(_data.TargetNumber);
        _list[3] = RLPEncode.encodeBytes(_toBytes(_data.TargetHash));
        return keccak256(RLPEncode.encodeList(_list));
    }

    function _getBlockHash(BlockHeader memory _header) internal pure returns (bytes32) {
        bytes[] memory _list;
        if(_header.parentBeaconBlockRoot == nilParentBeaconBlockRoot){
           _list = new bytes[](19);
        } else {
           _list = new bytes[](20);
        }
        _list[0] = RLPEncode.encodeBytes(_toBytes(_header.parentHash));
        _list[1] = RLPEncode.encodeBytes(_toBytes(_header.sha3Uncles));
        _list[2] = RLPEncode.encodeAddress(_header.miner);
        _list[3] = RLPEncode.encodeBytes(_toBytes(_header.stateRoot));
        _list[4] = RLPEncode.encodeBytes(_toBytes(_header.transactionsRoot));
        _list[5] = RLPEncode.encodeBytes(_toBytes(_header.receiptsRoot));
        _list[6] = RLPEncode.encodeBytes(_header.logsBloom);
        _list[7] = RLPEncode.encodeUint(_header.difficulty);
        _list[8] = RLPEncode.encodeUint(_header.number);
        _list[9] = RLPEncode.encodeUint(_header.gasLimit);
        _list[10] = RLPEncode.encodeUint(_header.gasUsed);
        _list[11] = RLPEncode.encodeUint(_header.timestamp);
        _list[12] = RLPEncode.encodeBytes(_header.extraData);
        _list[13] = RLPEncode.encodeBytes(_toBytes(_header.mixHash));
        _list[14] = RLPEncode.encodeBytes(_header.nonce);
        _list[15] = RLPEncode.encodeUint(_header.baseFeePerGas);
        _list[16] = RLPEncode.encodeBytes(_toBytes(_header.withdrawalsRoot));
        _list[17] = RLPEncode.encodeUint(_header.blobGasUsed);
        _list[18] = RLPEncode.encodeUint(_header.excessBlobGas);
        if( _list.length != 19 ) {
            _list[19] = RLPEncode.encodeBytes(_toBytes(_header.parentBeaconBlockRoot));
        }
        return keccak256(RLPEncode.encodeList(_list));
    }


    // getValidatorBytesFromHeader returns the validators bytes extracted from the header's extra field if exists.
    // The validators bytes would be contained only in the epoch block's header, and its each validator bytes length is fixed.
    // On luban fork, we introduce vote attestation into the header's extra field, so extra format is different from before.
    // Before luban fork: |---Extra Vanity---|---Validators Bytes (or Empty)---|---Extra Seal---|
    // After luban fork:  |---Extra Vanity---|---Validators Number and Validators Bytes (or Empty)---|---Vote Attestation (or Empty)---|---Extra Seal---|
    // After bohr fork:   |---Extra Vanity---|---Validators Number and Validators Bytes (or Empty)---|---Turn Length (or Empty)---|---Vote Attestation (or Empty)---|---Extra Seal---|
    function _getBLSPublicKey(bytes memory _extraData) internal pure returns (bytes[] memory res) {
        // 1 byte for validators num
        uint256 prefix = EXTRA_VANITY + EXTRASEAL + 1;
        uint256 keyLenght = ADDRESS_LENGTH + BLS_PUBLICKEY_LENGTH;
        require(_extraData.length > prefix, "invalid _extraData length");
        uint256 num;
        uint256 point;
        assembly {
            //skip 32 byte data length + 32 byte EXTRA_VANITY
            point := add(_extraData, 64)
            // 1 byte for validators num
            num := shr(248, mload(point))
        }
        require(_extraData.length >= (prefix + keyLenght * num), "invalid _extraData length");
        res = new bytes[](num);
        point += 1;
        for (uint i = 0; i < num; i++) {
          point += ADDRESS_LENGTH;
          res[i] = _memoryToBytes(point,BLS_PUBLICKEY_LENGTH);  
          point += BLS_PUBLICKEY_LENGTH;
        }
    }

    function _memoryToBytes(uint _ptr, uint _length) internal pure returns (bytes memory res) {
        if (_length != 0) {
            assembly {
                // 0x40 is the address of free memory pointer.
                res := mload(0x40)
                let end := add(
                    res,
                    and(add(_length, 63), 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0)
                )
                // end = res + 32 + 32 * ceil(length / 32).
                mstore(0x40, end)
                mstore(res, _length)
                let destPtr := add(res, 32)
                // prettier-ignore
                for { } 1 { } {
                    mstore(destPtr, mload(_ptr))
                    destPtr := add(destPtr, 32)
                    if eq(destPtr, end) {
                        break
                    }
                    _ptr := add(_ptr, 32)
                }
            }
        }
    }

    function _toBytes(bytes32 _b) internal pure returns(bytes memory) {
         return abi.encodePacked(_b);
    }

}