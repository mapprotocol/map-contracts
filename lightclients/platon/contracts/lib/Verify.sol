// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

import "./RLPReader.sol";
import "./RLPEncode.sol";
import "../interface/IMPTVerify.sol";

// import "hardhat/console.sol";

library Verify {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint256 internal constant ADDRESS_LENGTH = 20;

    uint256 internal constant EPOCH_NUM = 430;

    uint256 internal constant MIN_GAS_LIMIT = 5000;

    uint256 internal constant HASH_LENGTH = 32;

    uint256 internal constant VALIDATOR_LENGTH = 43;

    uint256 internal constant PUBKEY_LENGTH = 64;

    uint256 internal constant BLS_PUBKEY_LENGTH = 96;

    uint256 internal constant EXTRA_DATA_LENGHT = 97;

    struct BlockHeader {
        bytes parentHash;
        address miner;
        bytes stateRoot;
        bytes transactionsRoot;
        bytes receiptsRoot;
        bytes logsBloom;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 timestamp;
        bytes extraData;
        bytes nonce;
    }

    struct Validator {
        address Address;
        bytes NodeId;
        bytes BlsPubKey;
    }

    struct QuorumCert {
        uint256 epoch;
        uint256 viewNumber;
        bytes32 blockHash;
        uint256 blockNumber;
        uint256 blockIndex;
        bytes signature;
        uint256 validatorSignBitMap;
        uint256 signedCount;
    }

    struct ReceiptProof {
        TxReceipt txReceipt;
        bytes keyIndex;
        bytes[] proof;
    }

    struct TxReceipt {
        uint256 receiptType;
        bytes postStateOrStatus;
        uint256 cumulativeGasUsed;
        bytes bloom;
        TxLog[] logs;
    }

    struct TxLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    // function _recoverSigner(
    //     BlockHeader memory _header
    // ) internal pure returns (address) {
    //     (bytes32  splitHash, bytes memory signature) = _splitValidatorsHashFromExtra(
    //         _header.extraData
    //     );

    //     bytes32 hash = _getBlockHash(_header, abi.encodePacked(splitHash));

    //     bytes32 r;
    //     bytes32 s;
    //     uint8 v;
    //     // ecrecover takes the signature parameters, and the only way to get them
    //     // currently is to use assembly.
    //     assembly {
    //         r := mload(add(signature, 0x20))
    //         s := mload(add(signature, 0x40))
    //         v := byte(0, mload(add(signature, 0x60)))
    //     }
    //     if (v <= 1) {
    //         v = v + 27;
    //     }

    //     address signer = ecrecover(hash, v, r, s);
    //     return signer;
    // }

    function _validateHeader(
        BlockHeader memory _header
    ) internal pure returns (bool) {
        if (_header.extraData.length != EXTRA_DATA_LENGHT) {
            return false;
        }


        if (_header.gasUsed > _header.gasLimit) {
            return false;
        }

        // if(_header.miner != _recoverSigner(_header)){
        //     return false;
        // }

        return true;
    }

    function _getBlockHash(
        BlockHeader memory _header,
        bytes memory _extraData
    ) internal pure returns (bytes32) {
        bytes[] memory list = new bytes[](12);
        list[0] = RLPEncode.encodeBytes(_header.parentHash);
        list[1] = RLPEncode.encodeAddress(_header.miner);
        list[2] = RLPEncode.encodeBytes(_header.stateRoot);
        list[3] = RLPEncode.encodeBytes(_header.transactionsRoot);
        list[4] = RLPEncode.encodeBytes(_header.receiptsRoot);
        list[5] = RLPEncode.encodeBytes(_header.logsBloom);
        list[6] = RLPEncode.encodeUint(_header.number);
        list[7] = RLPEncode.encodeUint(_header.gasLimit);
        list[8] = RLPEncode.encodeUint(_header.gasUsed);
        list[9] = RLPEncode.encodeUint(_header.timestamp);
        list[10] = RLPEncode.encodeBytes(_extraData);
        list[11] = RLPEncode.encodeBytes(_header.nonce);
        return keccak256(RLPEncode.encodeList(list));
    }

    function _verifyValidators(
        Validator[] memory _validators,
        bytes memory _extraData
    ) internal pure returns (bool) {
        // if(_validators.length != VALIDATOR_LENGTH) {
        //     return false;
        // }

        bytes[] memory v = new bytes[](_validators.length);

        for (uint256 i = 0; i < _validators.length; i++) {
            if (
                _validators[i].NodeId.length != PUBKEY_LENGTH &&
                _validators[i].BlsPubKey.length != BLS_PUBKEY_LENGTH
            ) {
                return false;
            }
            bytes[] memory list = new bytes[](2);
            list[0] = RLPEncode.encodeBytes(_validators[i].NodeId);
            list[1] = RLPEncode.encodeBytes(_validators[i].BlsPubKey);
            v[i] = RLPEncode.encodeList(list);
        }
        bytes32 hash = keccak256(RLPEncode.encodeList(v));
        (bytes32 split, ) = _splitValidatorsHashFromExtra(_extraData);
        if (hash != split) {
            return false;
        }

        return true;
    }

    function _verifyQuorumCert(
        bytes32 _blockHash,
        QuorumCert memory _quorumCert,
        Validator[] memory _validators
    ) internal view returns (bool) {
        if (_blockHash != _quorumCert.blockHash) {
            return false;
        }
        // for bitmap validator must less than 256
        if (_validators.length >= 256) {
            return false;
        }
        // signed more than 2/3 of validators
        if (_quorumCert.signedCount < (_validators.length * 2) / 3) {
            return false;
        }
        bytes32 message = _getBlsSigMsg(_quorumCert);
        uint256 count;
        bytes[] memory pubkeys = new bytes[](_quorumCert.signedCount);
        uint256 bitmap = _quorumCert.validatorSignBitMap;
        for (uint256 i = 0; i < _validators.length; i++) {
            uint256 shift = 1 << i;
            if (bitmap & shift != 0) {
                pubkeys[count] = (_validators[i].BlsPubKey);
                count++;
                if (count == _quorumCert.signedCount) {
                    break;
                }
            }
        }

        if (count < _quorumCert.signedCount) {
            return false;
        }
        // VerifyAggSig
        return _verifyBlsAggSignature(message, pubkeys, _quorumCert.signature);
    }

    function _verifyBlsAggSignature(
        bytes32 _message,
        bytes[] memory _pubkeys,
        bytes memory signature
    ) internal view returns (bool) {
        // return BLS.fast_aggregate_verify(_pubkeys,_message,signature);
        return true;
    }

    // VerifyAggSig
    function _getBlsSigMsg(
        QuorumCert memory _q
    ) internal pure returns (bytes32) {
        bytes[] memory list = new bytes[](5);
        list[0] = RLPEncode.encodeUint(_q.epoch);
        list[1] = RLPEncode.encodeUint(_q.viewNumber);
        list[2] = RLPEncode.encodeBytes(abi.encodePacked(_q.blockHash));
        list[3] = RLPEncode.encodeUint(_q.blockNumber);
        list[4] = RLPEncode.encodeUint(_q.blockIndex);
        return keccak256(RLPEncode.encodeList(list));
    }

    function _validateProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mptVerify
    ) internal pure returns (bool success, bytes memory logs) {
        bytes memory bytesReceipt = _encodeReceipt(_receipt.txReceipt);
        bytes memory expectedValue = bytesReceipt;
        if (_receipt.txReceipt.receiptType > 0) {
            expectedValue = abi.encodePacked(
                bytes1(uint8(_receipt.txReceipt.receiptType)),
                bytesReceipt
            );
        }

        success = IMPTVerify(_mptVerify).verifyTrieProof(
            _receiptsRoot,
            _receipt.keyIndex,
            _receipt.proof,
            expectedValue
        );

        if (success) logs = bytesReceipt.toRlpItem().toList()[3].toRlpBytes(); // list length must be 4
    }

    function _encodeReceipt(
        TxReceipt memory _txReceipt
    ) internal pure returns (bytes memory output) {
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

    function _splitValidatorsHashFromExtra(
        bytes memory _extraData
    ) internal pure returns (bytes32, bytes memory) {
        bytes32 hash;
        uint256 ptr;
        assembly {
            ptr := _extraData
            //extraData =  ValidatorsHash 32byte + signature
            hash := mload(add(ptr, 32)) //memory bytes length
        }

        bytes memory signature = _memoryToBytes(
            ptr + 64, //memory bytes length +  ValidatorsHash length
            EXTRA_DATA_LENGHT - 32 // 32 -> ValidatorsHash length
        );
        return (hash, signature);
    }

    function _memoryToBytes(
        uint _ptr,
        uint _length
    ) internal pure returns (bytes memory res) {
        if (_length != 0) {
            assembly {
                // 0x40 is the address of free memory pointer.
                res := mload(0x40)
                let end := add(
                    res,
                    and(
                        add(_length, 63),
                        0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0
                    )
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
}
