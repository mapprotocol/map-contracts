// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/RLPEncode.sol";
import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";

library Verify {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint256 internal constant ADDRESS_LENGTH = 20;

    uint256 internal constant EXTRA_VANITY = 32;

    uint256 internal constant EPOCH_NUM = 200;

    uint256 internal constant EXTRASEAL = 65;

    uint256 internal constant MIN_GAS_LIMIT = 5000;

    uint256 internal constant BLS_PUBLICKEY_LENGTH = 48;

    uint256 internal constant TESTNET_LU_BAN_FORK_BLOCK = 29295050;

    uint256 internal constant TESTNET_LONDON_FORK_BLOCK = 31103030;

    uint256 internal constant MAINNET_LU_BAN_FORK_BLOCK = 29020050;

    uint256 internal constant MAINNET_LONDON_FORK_BLOCK = 31302048;

    uint256 internal constant MAINNET_CHAIN_ID = 56;

    bytes32 constant SHA3_UNCLES = 0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347;

    bytes8 constant NONCE = 0x0000000000000000;

    bytes32 constant MIX_HASH = 0x0000000000000000000000000000000000000000000000000000000000000000;

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
        uint256 baseFeePerGas;
        bytes withdrawalsRoot;
        uint256 blobGasUsed;
        uint256 excessBlobGas;
        bytes parentBeaconBlockRoot;
        bytes requestsHash;
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

    function _verifyHeaderSignature(BlockHeader memory _header, uint256 _chainId) internal pure returns (bool) {
        (bytes memory signature, bytes memory extraData) = _splitExtra(_header.extraData);
        bytes32 hash = _getSealHash(_header, extraData, _chainId);

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

        return signer == _header.miner;
    }

    function _validateHeader(
        BlockHeader memory _header,
        uint256 _parentGasLimit,
        uint256 _minEpochBlockExtraDataLen
    ) internal pure returns (bool) {
        if (_header.extraData.length < (EXTRA_VANITY + EXTRASEAL)) {
            return false;
        }
        //Epoch block
        if (_header.number % EPOCH_NUM == 0) {
            if (_header.extraData.length < _minEpochBlockExtraDataLen) {
                return false;
            }
        }

        // if (_header.difficulty != 2 && _header.difficulty != 1) {
        //     return false;
        // }

        if (_header.sha3Uncles.length != 32 || bytes32(_header.sha3Uncles) != SHA3_UNCLES) {
            return false;
        }

        if (_header.nonce.length != 8 || bytes8(_header.nonce) != NONCE) {
            return false;
        }

        if (_header.mixHash.length != 32 || bytes32(_header.mixHash) != MIX_HASH) {
            return false;
        }
        //2**63 - 1 maxGasLimit
        if (_header.gasLimit > 2 ** 63 - 1 || _header.gasLimit < _header.gasUsed) {
            return false;
        }

        uint256 diff = _parentGasLimit > _header.gasLimit
            ? _parentGasLimit - _header.gasLimit
            : _header.gasLimit - _parentGasLimit;
        //5000 minGasLimit
        if (diff >= _parentGasLimit / 256 || _header.gasLimit < MIN_GAS_LIMIT) {
            return false;
        }

        return true;
    }

    function _getSealHash(
        BlockHeader memory _header,
        bytes memory _extraData,
        uint256 _chainId
    ) internal pure returns (bytes32) {
        bytes[] memory list;
        if(bytes32(_header.parentBeaconBlockRoot) != bytes32("")){
            list = new bytes[](16);
        } else if(bytes32(_header.requestsHash) != bytes32("")){
            list = new bytes[](22);
        } else {
            list = new bytes[](21);
        }
        list[0] = RLPEncode.encodeUint(_chainId);
        _headerToList(_header, _extraData, list);
        return keccak256(RLPEncode.encodeList(list));
    }

    function _headerToList(
        BlockHeader memory _header,
        bytes memory _extraData,
        bytes[] memory _list
    ) internal pure {
        _list[1] = RLPEncode.encodeBytes(_header.parentHash);
        _list[2] = RLPEncode.encodeBytes(_header.sha3Uncles);
        _list[3] = RLPEncode.encodeAddress(_header.miner);
        _list[4] = RLPEncode.encodeBytes(_header.stateRoot);
        _list[5] = RLPEncode.encodeBytes(_header.transactionsRoot);
        _list[6] = RLPEncode.encodeBytes(_header.receiptsRoot);
        _list[7] = RLPEncode.encodeBytes(_header.logsBloom);
        _list[8] = RLPEncode.encodeUint(_header.difficulty);
        _list[9] = RLPEncode.encodeUint(_header.number);
        _list[10] = RLPEncode.encodeUint(_header.gasLimit);
        _list[11] = RLPEncode.encodeUint(_header.gasUsed);
        _list[12] = RLPEncode.encodeUint(_header.timestamp);
        _list[13] = RLPEncode.encodeBytes(_extraData);
        _list[14] = RLPEncode.encodeBytes(_header.mixHash);
        _list[15] = RLPEncode.encodeBytes(_header.nonce);
        if(_list.length != 16){
            _list[16] = RLPEncode.encodeUint(_header.baseFeePerGas);
            _list[17] = RLPEncode.encodeBytes(_header.withdrawalsRoot);
            _list[18] = RLPEncode.encodeUint(_header.blobGasUsed);
            _list[19] = RLPEncode.encodeUint(_header.excessBlobGas);
            _list[20] = RLPEncode.encodeBytes(_header.parentBeaconBlockRoot);
            if(_list.length != 21){
                _list[21] = RLPEncode.encodeBytes(_header.requestsHash);
            }
        } 
    }

    function _validateProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mptVerify
    ) internal pure returns (bool success, bytes memory logs) {
        bytes memory bytesReceipt = _encodeReceipt(_receipt.txReceipt);
        bytes memory expectedValue = bytesReceipt;
        if (_receipt.txReceipt.receiptType > 0) {
            expectedValue = abi.encodePacked(bytes1(uint8(_receipt.txReceipt.receiptType)), bytesReceipt);
        }

        success = IMPTVerify(_mptVerify).verifyTrieProof(
            _receiptsRoot,
            _receipt.keyIndex,
            _receipt.proof,
            expectedValue
        );

        if (success) logs = bytesReceipt.toRlpItem().toList()[3].toRlpBytes(); // list length must be 4
    }

    function _encodeReceipt(TxReceipt memory _txReceipt) internal pure returns (bytes memory output) {
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
        //extraData never less than 97
        extraData = _memoryToBytes(ptr, _extraData.length - EXTRASEAL);

        ptr += _extraData.length - EXTRASEAL;

        signature = _memoryToBytes(ptr, EXTRASEAL);
    }

    function _getValidators(
        uint256 _chainId,
        uint256 _blockNum,
        bytes memory _extraData
    ) internal pure returns (bytes memory) {
        if (_isAfterLuBanFork(_chainId, _blockNum)) {
            return _getValidatorsAfterLuBanFork(_extraData);
        } else {
            return _getValidatorsBeforeLuBanFork(_extraData);
        }
    }

    function _getValidatorsBeforeLuBanFork(bytes memory _extraData) internal pure returns (bytes memory) {
        require(_extraData.length > (EXTRA_VANITY + EXTRASEAL), "invalid _extraData length");

        require((_extraData.length - EXTRA_VANITY - EXTRASEAL) % ADDRESS_LENGTH == 0, "invalid _extraData length");
        uint256 ptr;
        assembly {
            ptr := _extraData
        }
        //skip 32 byte data length + 32 byte EXTRA_VANITY
        ptr += 64;
        //extraData never less than 97
        return _memoryToBytes(ptr, _extraData.length - (EXTRA_VANITY + EXTRASEAL));
    }

    // getValidatorBytesFromHeader returns the validators bytes extracted from the header's extra field if exists.
    // The validators bytes would be contained only in the epoch block's header, and its each validator bytes length is fixed.
    // On luban fork, we introduce vote attestation into the header's extra field, so extra format is different from before.
    // Before luban fork: |---Extra Vanity---|---Validators Bytes (or Empty)---|---Extra Seal---|
    // After luban fork:  |---Extra Vanity---|---Validators Number and Validators Bytes (or Empty)---|---Vote Attestation (or Empty)---|---Extra Seal---|
    function _getValidatorsAfterLuBanFork(bytes memory _extraData) internal pure returns (bytes memory res) {
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

        assembly {
            // 0x40 is the address of free memory pointer.
            res := mload(0x40)
            let length := mul(ADDRESS_LENGTH, num)
            //skip 32 byte data length
            let start := add(res, 32)
            // res end point
            let end := add(start, length)
            mstore(0x40, end)
            //store length for first 32 bytes
            mstore(res, length)
            //skip 1 byte for validators num
            point := add(point, 1)
            for {
                let i := 0
            } lt(i, num) {
                i := add(i, 1)
            } {
                // address lenth is 20 bytes Discard others 12 bytes
                let a := and(
                    mload(add(point, mul(i, keyLenght))),
                    0xffffffffffffffffffffffffffffffffffffffff000000000000000000000000
                )
                mstore(add(start, mul(i, ADDRESS_LENGTH)), a)
            }
        }
    }

    function _isAfterLuBanFork(uint256 _chainId, uint256 _blockNum) internal pure returns (bool) {
        if (_chainId == MAINNET_CHAIN_ID) {
            return MAINNET_LU_BAN_FORK_BLOCK > 0 && _blockNum > MAINNET_LU_BAN_FORK_BLOCK;
        } else {
            return _blockNum > TESTNET_LU_BAN_FORK_BLOCK;
        }
    }

    function _isAfterLondonFork(uint256 _chainId, uint256 _blockNum) internal pure returns (bool) {
        if (_chainId == MAINNET_CHAIN_ID) {
            return MAINNET_LONDON_FORK_BLOCK > 0 && _blockNum >= MAINNET_LONDON_FORK_BLOCK;
        } else {
            return _blockNum >= TESTNET_LONDON_FORK_BLOCK;
        }
    }

    function _containsValidator(bytes memory _validators, address _miner, uint256 _index) internal pure returns (bool) {
        uint256 m = uint256(uint160(_miner));

        uint256 ptr;
        assembly {
            ptr := _validators
        }
        // skip 32 byte data length
        ptr += 32;
        uint256 length = _validators.length / ADDRESS_LENGTH;
        for (uint256 i = 0; i < length; i++) {
            uint256 v;
            uint256 tem = ptr + ((_index + i) % length) * ADDRESS_LENGTH;
            assembly {
                v := mload(tem)
            }
            // 96bit => 12byte
            if (v >> 96 == m) {
                return true;
            }
        }

        return false;
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
}
