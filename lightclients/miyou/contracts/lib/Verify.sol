// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "./RLPReader.sol";
import "./RLPEncode.sol";
import "../interface/IMPTVerify.sol";


library Verify {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint256 internal constant ADDRESS_LENGTH = 20;

    uint256 internal constant EXTRA_VANITY = 32;

    uint256 internal constant EPOCH_NUM = 30000;

    uint256 internal constant EXTRASEAL = 65;

    bytes8 constant NONCE_DROP = 0x0000000000000000;
	
	bytes8 constant NONCE_AUTH = 0xffffffffffffffff;

    bytes32 constant MIX_HASH =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    struct BlockHeader {
        bytes parentHash;
        bytes sha3Uncles;
        address miner;
        bytes stateRoot;
        bytes transactionsRoot;
        bytes receiptsRoot;
        bytes logsBloom;
        uint256 difficulty;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 timestamp;
        bytes extraData;
        bytes mixHash;
        bytes nonce;
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

    function _recoverSigner(
        BlockHeader memory _header
    ) internal pure returns (address) {
        (bytes memory signature, bytes memory extraData) = _splitExtra(_header.extraData);
        bytes32 hash = keccak256(_encodeSigHeader(_header, extraData));

        bytes32 r;
        bytes32 s;
        uint8 v;
        // ecrecover takes the signature parameters, and the only way to get them
        // currently is to use assembly.
        assembly {
            r := mload(add(signature, 0x20))
            s := mload(add(signature, 0x40))
            v := byte(0, mload(add(signature, 0x60)))
        }
        if (v <= 1) {
            v = v + 27;
        }

        address signer = ecrecover(hash, v, r, s);

        return signer;
    }

    function _validateHeader(
        BlockHeader memory _header,
        uint256 _minEpochBlockExtraDataLen,
        BlockHeader memory _parent
    ) internal pure returns (bool) {
        if (_header.extraData.length < (EXTRA_VANITY + EXTRASEAL)) {
            return false;
        }

		if(_header.number > 0) {
			if (_header.difficulty != 1 && _header.difficulty != 2) {
				return false;
			}
		}

        if (_header.parentHash.length != 32) {
            return false;
        }

        if (
            _header.sha3Uncles.length != 32
        ) {
            return false;
        }

        if (_header.nonce.length != 8 || (bytes8(_header.nonce) != NONCE_AUTH && bytes8(_header.nonce) != NONCE_DROP)) {
            return false;
        }

        if (
            _header.mixHash.length != 32 || bytes32(_header.mixHash) != MIX_HASH
        ) {
            return false;
        }
		
		bool epochBlock = _header.number % EPOCH_NUM == 0;
		if (epochBlock) {
		    if (_header.extraData.length < _minEpochBlockExtraDataLen) {
		        return false;
		    }
			if (_header.nonce.length != 8 || bytes8(_header.nonce) != NONCE_DROP ) {
			    return false;
			}
			if ((_header.extraData.length - EXTRA_VANITY - EXTRASEAL) % ADDRESS_LENGTH != 0) {
				return false;
			}
		} else {
			if (_header.extraData.length - EXTRA_VANITY - EXTRASEAL != 0) {
				return false;
			}
		}
		if(_header.gasUsed > _header.gasLimit) {
			return false;
		}

        if (_header.number != _parent.number) {
            if (_header.timestamp <= _parent.timestamp) {
                return false;
            }
        }

        return true;
    }

    function _encodeSigHeader(
        BlockHeader memory _header,
        bytes memory _extraData
    ) internal pure returns (bytes memory output) {
        bytes[] memory list = new bytes[](15);
        list[0] = RLPEncode.encodeBytes(_header.parentHash);
        list[1] = RLPEncode.encodeBytes(_header.sha3Uncles);
        list[2] = RLPEncode.encodeAddress(_header.miner);
        list[3] = RLPEncode.encodeBytes(_header.stateRoot);
        list[4] = RLPEncode.encodeBytes(_header.transactionsRoot);
        list[5] = RLPEncode.encodeBytes(_header.receiptsRoot);
        list[6] = RLPEncode.encodeBytes(_header.logsBloom);
        list[7] = RLPEncode.encodeUint(_header.difficulty);
        list[8] = RLPEncode.encodeUint(_header.number);
        list[9] = RLPEncode.encodeUint(_header.gasLimit);
        list[10] = RLPEncode.encodeUint(_header.gasUsed);
        list[11] = RLPEncode.encodeUint(_header.timestamp);
        list[12] = RLPEncode.encodeBytes(_extraData);
        list[13] = RLPEncode.encodeBytes(_header.mixHash);
        list[14] = RLPEncode.encodeBytes(_header.nonce);
        output = RLPEncode.encodeList(list);
    }

    function _validateProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mptVerify
    ) internal view returns (bool success, bytes memory logs) {
        bytes memory bytesReceipt = _encodeReceipt(_receipt.txReceipt);
        bytes memory expectedValue = bytesReceipt;
        if (_receipt.txReceipt.receiptType > 0) {
            expectedValue = abi.encodePacked(bytes1(uint8(_receipt.txReceipt.receiptType)),bytesReceipt);
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
            bytes[] memory loglist1 = new bytes[]( _txReceipt.logs[j].topics.length);

            for (uint256 i = 0; i < _txReceipt.logs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes( _txReceipt.logs[j].topics[i]);
            }
            loglist[1] = RLPEncode.encodeList(loglist1);
            loglist[2] = RLPEncode.encodeBytes(_txReceipt.logs[j].data);
            bytes memory logBytes = RLPEncode.encodeList(loglist);
            listLog[j] = logBytes;
        }
        list[3] = RLPEncode.encodeList(listLog);
        output = RLPEncode.encodeList(list);
    }

    function _splitExtra(
        bytes memory _extraData
    ) internal pure returns (bytes memory signature, bytes memory extraData) {
        uint256 ptr;
        assembly {
            ptr := _extraData
        }
        // skip 32 byte data length
        ptr += 32;
        //extraData =  EXTRA_VANITY + (address)... + EXTRASEAL
        extraData = _memoryToBytes(ptr, _extraData.length - EXTRASEAL);

        ptr += _extraData.length - EXTRASEAL;

        signature = _memoryToBytes(ptr, EXTRASEAL);
    }

    function _getValidators(
        bytes memory _extraData
    ) internal pure returns (bytes memory) {
        require(
            _extraData.length > (EXTRA_VANITY + EXTRASEAL),
            "_extraData length too short"
        );

        require(
            (_extraData.length - EXTRA_VANITY - EXTRASEAL) %
            ADDRESS_LENGTH == 0,
            "invalid _extraData length"
        );
        uint256 ptr;
        assembly {
            ptr := _extraData
        }
        //skip EXTRA_VANITY + data length
        ptr += 64;
        //extraData =  EXTRA_VANITY + (address)... + EXTRASEAL
        uint256 length = (_extraData.length - (EXTRA_VANITY + EXTRASEAL)) /
            (ADDRESS_LENGTH);
        bytes memory result;
        for (uint256 i = 0; i < length; i++) {
            bytes32 v;
            uint256 tem = ptr + i * ADDRESS_LENGTH;
            assembly {
                v := mload(tem)
            }
            result = abi.encodePacked(result, bytes20(v));
        }

        return result;
    }

    function _containsValidator(
        bytes memory _validators,
        address _miner
    ) internal pure returns (bool, uint256) {
        uint256 m = uint256(uint160(_miner));

        uint256 ptr;
        assembly {
            ptr := _validators
        }
        ptr += 32;
        uint256 length = _validators.length / ADDRESS_LENGTH;
        for (uint256 i = 0; i < length; i++) {
            uint256 v;
            uint256 tem = ptr + i * ADDRESS_LENGTH;
            assembly {
                v := mload(tem)
            }

            if (v >> 96 == m) {
                return (true, i);
            }
        }

        return (false, 0);
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
