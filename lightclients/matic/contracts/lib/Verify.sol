// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "./RLPReader.sol";
import "./RLPEncode.sol";
import "../interface/IMPTVerify.sol";
// import "hardhat/console.sol";

library Verify {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    bytes32 constant SHA3_UNCLES =
        0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347;

    bytes8 constant NONCE = 0x0000000000000000;

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
        uint256 baseFeePerGas;
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

    function recoverSigner(BlockHeader memory _header)
        internal
        pure
        returns (address)
    {
        (bytes memory signature, bytes memory extraData) = splitExtra(
            _header.extraData
        );

        bytes32 hash = keccak256(encodeSigHeader(_header, extraData));

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

    function validateHeader(
        BlockHeader memory _header,
        uint256 _minEpochBlockExtraDataLen
    ) internal pure returns (bool) {
        if (_header.extraData.length < 97) {
            return false;
        }
        //Epoch block
        if ((_header.number + 1) % 64 == 0) {
            if (_header.extraData.length < _minEpochBlockExtraDataLen) {
                return false;
            }
        }

        if (_header.difficulty > 200) {
            return false;
        }

        if (_header.parentHash.length != 32) {
            return false;
        }

        if (_header.miner != address(0)) {
            return false;
        }

        if (
            _header.sha3Uncles.length != 32 ||
            bytes32(_header.sha3Uncles) != SHA3_UNCLES
        ) {
            return false;
        }

        if (_header.nonce.length != 8 || bytes8(_header.nonce) != NONCE) {
            return false;
        }

        if (_header.mixHash.length != 32 || bytes32(_header.mixHash) != MIX_HASH) {
            return false;
        }
        //2**63 - 1 maxGasLimit minGasLimit 5000
        if (
            _header.gasLimit > 2**63 - 1 ||
            _header.gasLimit < 5000 ||
            _header.gasLimit < _header.gasUsed
        ) {
            return false;
        }

        return true;
    }

    function encodeSigHeader(BlockHeader memory _header, bytes memory _extraData)
        internal
        pure
        returns (bytes memory output)
    {
        bytes[] memory list = new bytes[](16);
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
        list[15] = RLPEncode.encodeUint(_header.baseFeePerGas);
        output = RLPEncode.encodeList(list);
    }

    function validateProof(
        bytes32 _receiptsRoot,
        ReceiptProof memory _receipt,
        address _mptVerify
    ) internal pure returns (bool success, bytes memory logs) {
        bytes memory bytesReceipt = encodeReceipt(_receipt.txReceipt);
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

        if (success)
            logs = bytesReceipt.toRlpItem().safeGetItemByIndex(3).toRlpBytes();
    }

    function encodeReceipt(TxReceipt memory _txReceipt)
        public
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

    function splitExtra(bytes memory _extraData)
        internal
        pure
        returns (bytes memory signature, bytes memory extraData)
    {
        uint256 ptr;
        assembly {
            ptr := _extraData
        }

        ptr += 32;
        //extraData never less than 97
        extraData = memoryToBytes(ptr, _extraData.length - 65);

        ptr += _extraData.length - 65;

        signature = memoryToBytes(ptr, 65);
    }

    function getValidators(bytes memory _extraData)
        internal
        pure
        returns (bytes memory)
    {
        uint256 ptr;
        assembly {
            ptr := _extraData
        }
        ptr += 64;
        uint256 legth = (_extraData.length - 97) / 40;
        bytes memory result;
        for (uint256 i = 0; i < legth; i++) {
            bytes32 v;
            uint256 tem = ptr + i * 40;
            assembly {
                v := mload(tem)
            }
            result = abi.encodePacked(result, bytes20(v));
        }

        return result;
    }

    function containValidator(bytes memory _validators, address _miner)
        internal
        pure
        returns (bool)
    {
        uint256 m = uint256(uint160(_miner));

        uint256 ptr;
        assembly {
            ptr := _validators
        }
        ptr += 32;
        uint256 length = _validators.length / 20;
        for (uint256 i = 0; i < length; i++) {
            uint256 v;
            uint256 tem = ptr + i * 20;
            assembly {
                v := mload(tem)
            }

            if (v >> 96 == m) {
                return true;
            }
        }

        return false;
    }

    function memoryToBytes(uint _ptr, uint _length)
        internal
        pure
        returns (bytes memory res)
    {
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
