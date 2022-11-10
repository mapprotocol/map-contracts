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

    function recoverSigner(BlockHeader memory header)
        internal
        pure
        returns (address)
    {
        (bytes memory signature, bytes memory extraData) = splitExtra(
            header.extraData
        );

        bytes32 hash = keccak256(encodeSigHeader(header, extraData));

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
        BlockHeader memory header,
        uint256 minEpochBlockExtraDataLen
    ) internal pure returns (bool) {
        if (header.extraData.length < 97) {
            return false;
        }
        //Epoch block
        if ((header.number + 1) % 64 == 0) {
            if (header.extraData.length < minEpochBlockExtraDataLen) {
                return false;
            }
        }

        if (header.difficulty > 200) {
            return false;
        }

        if (header.parentHash.length != 32) {
            return false;
        }

        if (header.miner != address(0)) {
            return false;
        }

        if (
            header.sha3Uncles.length != 32 ||
            bytes32(header.sha3Uncles) != SHA3_UNCLES
        ) {
            return false;
        }

        if (header.nonce.length != 8 || bytes8(header.nonce) != NONCE) {
            return false;
        }

        if (header.mixHash.length != 32 || bytes32(header.mixHash) != MIX_HASH) {
            return false;
        }
        //2**63 - 1 maxGasLimit minGasLimit 5000
        if (
            header.gasLimit > 2**63 - 1 ||
            header.gasLimit < 5000 ||
            header.gasLimit < header.gasUsed
        ) {
            return false;
        }

        return true;
    }

    function encodeSigHeader(BlockHeader memory header, bytes memory extraData)
        internal
        pure
        returns (bytes memory output)
    {
        bytes[] memory list = new bytes[](16);
        list[0] = RLPEncode.encodeBytes(header.parentHash);
        list[1] = RLPEncode.encodeBytes(header.sha3Uncles);
        list[2] = RLPEncode.encodeAddress(header.miner);
        list[3] = RLPEncode.encodeBytes(header.stateRoot);
        list[4] = RLPEncode.encodeBytes(header.transactionsRoot);
        list[5] = RLPEncode.encodeBytes(header.receiptsRoot);
        list[6] = RLPEncode.encodeBytes(header.logsBloom);
        list[7] = RLPEncode.encodeUint(header.difficulty);
        list[8] = RLPEncode.encodeUint(header.number);
        list[9] = RLPEncode.encodeUint(header.gasLimit);
        list[10] = RLPEncode.encodeUint(header.gasUsed);
        list[11] = RLPEncode.encodeUint(header.timestamp);
        list[12] = RLPEncode.encodeBytes(extraData);
        list[13] = RLPEncode.encodeBytes(header.mixHash);
        list[14] = RLPEncode.encodeBytes(header.nonce);
        list[15] = RLPEncode.encodeUint(header.baseFeePerGas);
        output = RLPEncode.encodeList(list);
    }

    function validateProof(
        bytes32 receiptsRoot,
        ReceiptProof memory receipt,
        address mptVerify
    ) internal pure returns (bool success, bytes memory logs) {
        bytes memory bytesReceipt = encodeReceipt(receipt.txReceipt);
        bytes memory expectedValue = bytesReceipt;
        if (receipt.txReceipt.receiptType > 0) {
            expectedValue = abi.encodePacked(
                bytes1(uint8(receipt.txReceipt.receiptType)),
                bytesReceipt
            );
        }

        success = IMPTVerify(mptVerify).verifyTrieProof(
            receiptsRoot,
            receipt.keyIndex,
            receipt.proof,
            expectedValue
        );

        if (success)
            logs = bytesReceipt.toRlpItem().safeGetItemByIndex(3).toBytes();
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

    function splitExtra(bytes memory extraData)
        internal
        pure
        returns (bytes memory _signature, bytes memory _extraData)
    {
        uint256 ptr;
        assembly {
            ptr := extraData
        }

        ptr += 32;
        //extraData never less than 97
        _extraData = memoryToBytes(ptr, extraData.length - 65);

        ptr += extraData.length - 65;

        _signature = memoryToBytes(ptr, 65);
    }

    function getValidators(bytes memory extraData)
        internal
        pure
        returns (bytes memory)
    {
        uint256 ptr;
        assembly {
            ptr := extraData
        }
        ptr += 64;
        uint256 legth = (extraData.length - 97) / 40;
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

    function containValidator(bytes memory validators, address miner)
        internal
        pure
        returns (bool)
    {
        uint256 m = uint256(uint160(miner));

        uint256 ptr;
        assembly {
            ptr := validators
        }
        ptr += 32;
        uint256 length = validators.length / 20;
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

    function memoryToBytes(uint ptr, uint length)
        internal
        pure
        returns (bytes memory res)
    {
        if (length != 0) {
            assembly {
                // 0x40 is the address of free memory pointer.
                res := mload(0x40)
                let end := add(
                    res,
                    and(
                        add(length, 63),
                        0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0
                    )
                )
                // end = res + 32 + 32 * ceil(length / 32).
                mstore(0x40, end)
                mstore(res, length)
                let destPtr := add(res, 32)
                // prettier-ignore
                for { } 1 { } {
                    mstore(destPtr, mload(ptr))
                    destPtr := add(destPtr, 32)
                    if eq(destPtr, end) {
                        break
                    }
                    ptr := add(ptr, 32)
                }
            }
        }
    }
}
