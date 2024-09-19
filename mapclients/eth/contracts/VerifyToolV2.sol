// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";
import "@mapprotocol/protocol/contracts/lib/LibRLP.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./interface/IVerifyToolV2.sol";

contract VerifyToolV2 is IVerifyToolV2 {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    /*
     3 bytes: RLP header
    33 bytes: parentHash;
    21 bytes: coinbase;
    33 bytes: root;
    33 bytes: txHash;
    33 bytes: receiptHash;
    259 bytes: bloom;
    1-9 bytes: number;
    1-9 bytes: gasLimit;
    1-9 bytes: gasUsed;
    1-9 bytes: time;
    n bytes:   extraData;
    33 bytes: mixDigest;
    33 bytes: nonce;
    1-9 bytes: baseFee;
    */
    uint256 internal constant COINBASE_OFFSET = 35;
    uint256 internal constant RECEIPT_ROOT_OFFSET = 123;
    uint256 internal constant BLOCK_NUMBER_OFFSET = 415;

    function verifyTrieProof(
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure override returns (bool success, bytes memory logs) {
        bytes32 expectedHash = keccak256(_receiptRlp);
        success = MPT.verify(expectedHash, _keyIndex, _proof, _receiptHash);
        if (success) {
            uint256 offset = (_receiptType == 0) ? 0 : 1;
            RLPReader.RLPItem memory rlpItem = _receiptRlp.toRlpItem(offset);
            logs = rlpItem.safeGetItemByIndex(3).unsafeToRlpBytes();
        }
        return (success, logs);
    }

    function verifyTrieProofWithLog(
        uint256 _logIndex,
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure override returns (bool success, ILightNode.txLog memory log) {
        bytes32 expectedHash = keccak256(_receiptRlp);
        success = MPT.verify(expectedHash, _keyIndex, _proof, _receiptHash);
        if (success) {
            uint256 offset = (_receiptType == 0) ? 0 : 1;
            RLPReader.RLPItem memory rlpItem = _receiptRlp.toRlpItem(offset);
            RLPReader.RLPItem memory logs = rlpItem.safeGetItemByIndex(3);
            log = _decodeTxLog(logs.safeGetItemByIndex(_logIndex));
        }
        return (success, log);
    }

    function _decodeTxLog(RLPReader.RLPItem memory item) private pure returns (ILightNode.txLog memory _txLog) {
        RLPReader.RLPItem[] memory items = item.toList();
        require(items.length >= 3, "log length to low");
        RLPReader.RLPItem[] memory firstItemList = items[1].toList();
        bytes32[] memory topic = new bytes32[](firstItemList.length);
        for (uint256 j = 0; j < firstItemList.length; j++) {
            topic[j] = firstItemList[j].toBytes32();
        }
        _txLog = ILightNode.txLog({addr: items[0].toAddress(), topics: topic, data: items[2].unsafeToBytes()});
    }

    function checkHeader(
        uint256 _blockNumber,
        bytes memory _header,
        bytes memory _signHeader,
        IVerifyToolV2.istanbulExtra memory ist,
        bool checkValidator,
        bool getReceiptRoot
    ) external override pure returns (bool success, string memory message, address coinbase, bytes32 receiptRoot) {
        // check block number
        RLPReader.RLPItem memory numberItem = _header.toRlpItem(BLOCK_NUMBER_OFFSET);
        uint256 number = numberItem.toUint();
        if (_blockNumber != number) {
            return (false, "Invalid block number", coinbase, receiptRoot);
        }

        // get extra item
        uint256 offset = BLOCK_NUMBER_OFFSET;
        RLPReader.RLPItem memory item = numberItem;
        for (uint256 i = 0; i < 4; i++) {
            offset += item.len;
            item = _header.toRlpItem(offset);
        }

        // check before ext
        if (!checkBeforeExt(_header, _signHeader, offset)) {
            return (false, "Invalid header", coinbase, receiptRoot);
        }

        // todo: check after ext

        // check extra data
        if (item.len < 113) {
            // min istanbul extra data
            return (false, "Invalid extra len", coinbase, receiptRoot);
        }
        // skip 32 bytes extra data and the istanbulExtra rlp header
        RLPReader.RLPItem memory istItem = _header.toRlpItem(offset + 2 + 0x20);
        if (checkValidator) {
            if (!checkIst(istItem, ist))
            {
                return (false, "Invalid istExt", coinbase, receiptRoot);
            }
        } else {
            RLPReader.RLPItem memory sealItem = istItem.safeGetItemByIndex(4);
            if (RLPReader.payloadKeccak256(sealItem) != keccak256(ist.seal)) {
                return (false, "Invalid seal", coinbase, receiptRoot);
            }
        }

        item = _header.toRlpItem(COINBASE_OFFSET);
        coinbase = item.toAddress();

        if (getReceiptRoot) {
            item = _header.toRlpItem(RECEIPT_ROOT_OFFSET);
            receiptRoot = item.toBytes32();
        }

        return (true, "", coinbase, receiptRoot);
    }

    function checkBeforeExt(
        bytes memory _header,
        bytes memory _signHeader,
        uint256 offset
    ) internal pure returns (bool) {
        uint256 memPtr;
        bytes32 result1;
        bytes32 result2;
        assembly {
            memPtr := add(_header, 0x23)
            result1 := keccak256(memPtr, offset)
            memPtr := add(_signHeader, 0x23)
            result2 := keccak256(memPtr, offset)
        }

        return (result1 == result2);
    }


    function checkEmptyIst(
        RLPReader.RLPItem memory ext,
        istanbulExtra memory ist
    ) internal pure returns (bool) {
        uint256 ist32;
        uint256 memPtr = ext.memPtr;
        assembly {
            ist32 := mload(memPtr)
        }

        if ((ist32 >> 28) != 0xc0c0c000) {
            return false;
        }

        RLPReader.RLPItem memory removeItem = ext.safeGetItemByIndex(3);
        uint256 removeList = removeItem.toUint();
        if (removeList != ist.removeList) {
            return false;
        }
        if (removeItem.memPtr - ext.memPtr != 4) {
            return false;
        }

        return true;
    }


    function checkIst(
        RLPReader.RLPItem memory ext,
        istanbulExtra memory ist
    ) internal pure returns (bool) {
        if (ist.validators.length == 0 && ist.removeList == 0) {
            //
            return checkEmptyIst(ext, ist);
        }

        RLPReader.RLPItem[] memory istList = ext.toList();

        uint256 removeList = istList[3].toUint();
        if (removeList != ist.removeList) {
            return false;
        }

        LibRLP.List[3] memory list;
        for (uint256 i = 0; i < 3; i++) {
            list[i] = LibRLP.l();
        }

        for (uint256 i = 0; i < ist.validators.length; i++) {
            LibRLP.p(list[0], ist.validators[i]);
            LibRLP.p(list[1], ist.addedPubKey[i]);
            LibRLP.p(list[2], ist.addedG1PubKey[i]);
        }
        for (uint256 i = 0; i < 3; i++) {
            bytes memory listBytes = LibRLP.encode(list[i]);
            if (RLPReader.rlpBytesKeccak256(istList[i]) != keccak256(listBytes)) {
                return false;
            }
        }

        return true;
    }


    function unsafeDecodeTxReceipt(bytes memory _receiptRlp) external pure override returns (bytes memory logHash) {
        return _receiptRlp.toRlpItem().safeGetItemByIndex(3).unsafeToRlpBytes();
    }


    function verifyHeaderHash(
        address _coinbase,
        bytes memory _seal,
        bytes32 headerBytesHash
    ) external pure override returns (bool ret) {
        bytes32 headerHash = keccak256(abi.encodePacked(headerBytesHash));
        ret = verifySign(_seal, headerHash, _coinbase);
    }


    function verifySign(bytes memory seal, bytes32 hash, address coinbase) internal pure returns (bool) {
        (bytes32 r, bytes32 s, uint8 v) = splitSignature(seal);
        if (v <= 1) {
            v = v + 27;
        }
        return coinbase == ECDSA.recover(hash, v, r, s);
    }

    function splitSignature(bytes memory sig) internal pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "invalid signature length");
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
    }
}
