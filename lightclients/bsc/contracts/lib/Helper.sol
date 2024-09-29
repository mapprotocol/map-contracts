// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/LibRLP.sol";
import "@mapprotocol/protocol/contracts/lib/LogDecode.sol";
import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";
import "@mapprotocol/protocol/contracts/interface/ILightVerifier.sol";
import {
    BlockHeader,
    ReceiptProof,
    VoteData,
    VoteAttestation,
    UpdateHeader
} from "./Types.sol";

import { BLS, Bytes, Memory } from "./bls/BLS.sol";

library Helper {
    using Memory for bytes32;
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
        address _mpt
    ) internal pure returns (bool success, bytes memory logs) {
        success = _mptVerify(_receiptsRoot, _receipt, _mpt);
        if (success) logs = LogDecode.getLogsFromTypedReceipt(_receipt.receiptType, _receipt.txReceipt); 
    }

    function _validateProofWithLog(
        uint256 _logIndex,
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mpt
    ) internal pure returns (bool success, ILightVerifier.txLog memory log) {
        success = _mptVerify(_receiptsRoot, _receipt, _mpt);
        if (success) log = LogDecode.decodeTxLogFromTypedReceipt(_logIndex, _receipt.receiptType, _receipt.txReceipt);
    }


    function _mptVerify(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mpt
    ) internal pure returns (bool success) {
        bytes32 expectedValue = keccak256(_receipt.txReceipt);
        success = IMPTVerify(_mpt).verifyTrieProof(
            _receiptsRoot,
            expectedValue,
            _receipt.keyIndex,
            _receipt.proof
        );
    }


    function _checkUpdateHeader(UpdateHeader memory _updateHeader) internal pure {
        bytes32 hash;
        BlockHeader[] memory headers = _updateHeader.headers;
        uint256 len = headers.length;
        for (uint i = 0; i < len; i++) {
            if(i != 0) {
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
        require(hash == votes[0].Data.SourceHash || hash == votes[0].Data.TargetHash, "invalid headers");
    }

    function _verifyVoteAttestation(VoteAttestation memory _vote, bytes memory _BLSPublicKeys) internal view {
        uint256 len = _getBLSPublicKeyCount(_BLSPublicKeys);
        require(len != 0, "empty BLSPublicKeys");
        uint64 voteAddressSet = _vote.VoteAddressSet;

        bytes[] memory uncompressPublicKeys = _vote.uncompressPublicKeys;
        uint256 count = uncompressPublicKeys.length;
        uint256 threshold = len * 2 / 3;
        require(count >= threshold, "not enough voted");
        uint64 mask = 1;
        uint256 index;
        for (uint256 i = 0; i < len; i++) {
             
            if((voteAddressSet & mask) != 0){
                bytes memory compressed = BLS.g1_compress(uncompressPublicKeys[index]);
                require(compressed.equals(_getBLSPublicKeyByIndex(_BLSPublicKeys, i)), "invalid uncompressPublicKey");
                index ++;
            }
            mask = mask << 1;
        }
        require(count == index, "uncompressPublicKeys number does not match");
        bytes32 voteDataRlpHash = _getVoteDataRlpHash(_vote.Data);
        
        _fastAggregateVerify(uncompressPublicKeys, _vote.Signature, voteDataRlpHash);
    }


    function _fastAggregateVerify(bytes[] memory _BLSPublicKeys, bytes memory _signature, bytes32 _voteDataRlpHash) internal view {
        require(BLS.fast_aggregate_verify(_BLSPublicKeys, _voteDataRlpHash, _signature), "bls pairing failed");
    }


    function _getVoteDataRlpHash(VoteData memory _data) internal pure returns(bytes32) {
        LibRLP.List memory list = LibRLP.l();
        LibRLP.p(list, _data.SourceNumber);
        LibRLP.p(list, _data.SourceHash.toBytes());
        LibRLP.p(list, _data.TargetNumber);
        LibRLP.p(list, _data.TargetHash.toBytes());
        return keccak256(LibRLP.encode(list));
    }

    function _getBlockHash(BlockHeader memory _header) internal pure returns (bytes32) {
        LibRLP.List memory list = LibRLP.l();
        LibRLP.p(list, _header.parentHash.toBytes());
        LibRLP.p(list, _header.sha3Uncles.toBytes());
        LibRLP.p(list, _header.miner);
        LibRLP.p(list, _header.stateRoot.toBytes());
        LibRLP.p(list, _header.transactionsRoot.toBytes());
        LibRLP.p(list, _header.receiptsRoot.toBytes());
        LibRLP.p(list, _header.logsBloom);
        LibRLP.p(list, _header.difficulty);
        LibRLP.p(list, _header.number);
        LibRLP.p(list, _header.gasLimit);
        LibRLP.p(list, _header.gasUsed);
        LibRLP.p(list, _header.timestamp);
        LibRLP.p(list, _header.extraData);
        LibRLP.p(list, _header.mixHash.toBytes());
        LibRLP.p(list, _header.nonce);
        LibRLP.p(list, _header.baseFeePerGas);
        LibRLP.p(list, _header.withdrawalsRoot.toBytes());
        LibRLP.p(list, _header.blobGasUsed);
        LibRLP.p(list, _header.excessBlobGas);
        if( _header.parentBeaconBlockRoot != nilParentBeaconBlockRoot ) {
            LibRLP.p(list, _header.parentBeaconBlockRoot.toBytes());
        }
        return keccak256(LibRLP.encode(list));
    }


    // getValidatorBytesFromHeader returns the validators bytes extracted from the header's extra field if exists.
    // The validators bytes would be contained only in the epoch block's header, and its each validator bytes length is fixed.
    // On luban fork, we introduce vote attestation into the header's extra field, so extra format is different from before.
    // Before luban fork: |---Extra Vanity---|---Validators Bytes (or Empty)---|---Extra Seal---|
    // After luban fork:  |---Extra Vanity---|---Validators Number and Validators Bytes (or Empty)---|---Vote Attestation (or Empty)---|---Extra Seal---|
    // After bohr fork:   |---Extra Vanity---|---Validators Number and Validators Bytes (or Empty)---|---Turn Length (or Empty)---|---Vote Attestation (or Empty)---|---Extra Seal---|
    function _getBLSPublicKey(bytes memory _extraData) internal pure returns (bytes memory res) {
        // 1 byte for validators num
        uint256 prefix = EXTRA_VANITY + EXTRASEAL + 1;
        uint256 keyLenght = ADDRESS_LENGTH + BLS_PUBLICKEY_LENGTH;
        require(_extraData.length > prefix, "invalid extraData length");
        uint256 num;
        uint256 point;
        assembly {
            //skip 32 byte data length + 32 byte EXTRA_VANITY
            point := add(_extraData, 64)
            // 1 byte for validators num
            num := shr(248, mload(point))
        }
        require(_extraData.length >= (prefix + keyLenght * num), "invalid extraData length");
        point += 1;
        for (uint i = 0; i < num; i++) {
            point += ADDRESS_LENGTH;
            res = res.concat(Memory.toBytes(point, BLS_PUBLICKEY_LENGTH));  
            point += BLS_PUBLICKEY_LENGTH;
        }
    }

    function _getBLSPublicKeyCount(bytes memory _keys) internal pure returns (uint256 res) {
        res = _keys.length / BLS_PUBLICKEY_LENGTH;
    }

    function _getBLSPublicKeyByIndex(bytes memory _keys, uint256 _index) internal pure returns (bytes memory res) {
        res = _keys.substr((_index * BLS_PUBLICKEY_LENGTH), BLS_PUBLICKEY_LENGTH);
    }

}