// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";
import "@mapprotocol/protocol/contracts/lib/LibRLP.sol";
import "@mapprotocol/protocol/contracts/lib/LogDecode.sol";
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
    9 bytes: nonce;
    1-9 bytes: baseFee;
    */
    uint256 internal constant COINBASE_OFFSET = 36;
    uint256 internal constant RECEIPT_ROOT_OFFSET = 123;
    uint256 internal constant BLOCK_NUMBER_OFFSET = 415;

    /*
    32 bytes: extra data;
    2/3 bytes: istanbulExtra RLP header;
    1-n bytes: added validators;
    1-n bytes: added public key;
    1-n bytes: added G1 key;
    1-33 bytes: remove list;
    1/67 bytes: seal;
    4 bytes: empty aggregated seal;
    70-132 bytes: parent aggregated seal;
    */
    uint256 internal constant MIN_EXTRA_LENGTH = 113;

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
    ) external pure override returns (bool success, ILightVerifier.txLog memory log) {
        bytes32 expectedHash = keccak256(_receiptRlp);
        success = MPT.verify(expectedHash, _keyIndex, _proof, _receiptHash);
        if (success) {
            log = LogDecode.decodeTxLogFromTypedReceipt(_logIndex, _receiptType, _receiptRlp);
        }
        return (success, log);
    }

    function checkHeader(
        uint256 _blockNumber,
        bytes memory _aggHeader,
        bytes memory _signHeader,
        IVerifyToolV2.istanbulExtra memory ist,
        bool checkValidator,
        bool getReceiptRoot
    ) external override pure returns (bool success, string memory message, address coinbase, bytes32 receiptRoot) {
        // check block number
        RLPReader.RLPItem memory item = _aggHeader.toRlpItem(BLOCK_NUMBER_OFFSET);
        uint256 number = item.toUint();
        if (_blockNumber != number) {
            return (false, "Invalid block number", coinbase, receiptRoot);
        }

        uint256 offset;
        (success, message, offset) = checkExtraData(checkValidator, _aggHeader, item, ist);
        if (!success) {
            return (success, message, coinbase, receiptRoot);
        }

        // check bytes before extra data
        if (!checkBeforeExt(_aggHeader, _signHeader, offset - 3)) {
            return (false, "Invalid header", coinbase, receiptRoot);
        }

        // todo: check after ext

        item = _aggHeader.toRlpItem(COINBASE_OFFSET);
        coinbase = item.toAddress();

        if (getReceiptRoot) {
            item = _aggHeader.toRlpItem(RECEIPT_ROOT_OFFSET);
            receiptRoot = item.toBytes32();
        }

        return (true, "", coinbase, receiptRoot);
    }

    function checkBeforeExt(
        bytes memory _header,
        bytes memory _signHeader,
        uint256 lengthBeforeExtra
    ) internal pure returns (bool) {
        uint256 memPtr;
        bytes32 result1;
        bytes32 result2;
        assembly {
            memPtr := add(_header, 0x23)  // skip RLP header
            result1 := keccak256(memPtr, lengthBeforeExtra)

            memPtr := add(_signHeader, 0x23) // skip RLP header
            result2 := keccak256(memPtr, lengthBeforeExtra)
        }

        return (result1 == result2);
    }

    function checkExtraData(
        bool checkValidator,
        bytes memory _header,
        RLPReader.RLPItem memory numberItem,
        IVerifyToolV2.istanbulExtra memory ist
    ) internal pure returns (bool success, string memory message, uint256 extraOffset) {
        // get extra item
        RLPReader.RLPItem memory item = numberItem;
        uint256 offset = BLOCK_NUMBER_OFFSET;
        // skip blockNumber, gasLimit, gasUsed and timestamp
        for (uint256 i = 0; i < 4; i++) {
            offset += item.len;
            item = _header.toRlpItem(offset);
        }

        // check extra data
        if (item.len < MIN_EXTRA_LENGTH) {
            // min istanbul extra data
            return (false, "Invalid extra len", 0);
        }
        // skip 32 bytes extra data and the istanbulExtra rlp header
        uint256 istOffset = (item.len > 0xFF) ? (3 + 0x20) : (2 + 0x20);
        RLPReader.RLPItem memory istItem = _header.toRlpItem(istOffset + offset);
        if (checkValidator) {
            if (!checkIst(istItem, ist))
            {
                return (false, "Invalid istExt", 0);
            }
        } else {
            RLPReader.RLPItem memory sealItem = istItem.safeGetItemByIndex(4);
            if (sealItem.payloadKeccak256() != keccak256(ist.seal)) {
                return (false, "Invalid seal", 0);
            }
        }

        return (true, "", offset);
    }


    // no added and removed validators
    // the istanbulExtra payload start with 0xC0C0C080
    function checkEmptyIst(
        RLPReader.RLPItem memory istItem
    ) internal pure returns (bool) {
        uint256 ist32;
        (uint256 memPtr, ) = istItem.payloadLocation();
        assembly {
            ist32 := mload(memPtr)
        }
        if ((ist32 >> 28 * 8) != 0xc0c0c080) {
            return false;
        }

        return true;
    }


    function checkIst(
        RLPReader.RLPItem memory istItem,
        IVerifyToolV2.istanbulExtra memory ist
    ) internal pure returns (bool) {
        if (ist.validators.length == 0 && ist.removeList == 0) {
            //
            return checkEmptyIst(istItem);
        }

        RLPReader.RLPItem[] memory istList = istItem.toList();

        LibRLP.List[3] memory list;
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

        uint256 removeList = istList[3].toUint();
        if (removeList != ist.removeList) {
            return false;
        }

        if (istList[4].payloadKeccak256() != keccak256(ist.seal)) {
            return false;
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
