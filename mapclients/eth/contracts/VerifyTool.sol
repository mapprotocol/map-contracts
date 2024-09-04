// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";
import "@mapprotocol/protocol/contracts/lib/LibRLP.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./interface/IVerifyTool.sol";

contract VerifyTool is IVerifyTool {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    function getVerifyTrieProof(
        bytes32 _receiptHash,
        bytes memory _keyIndex,
        bytes[] memory _proof,
        bytes memory _receiptRlp,
        uint256 _receiptType
    ) external pure override returns (bool success, string memory message) {
        bytes memory expectedValue = getVerifyExpectedValueHash(_receiptType, _receiptRlp);
        success = MPT.verify(expectedValue, _keyIndex, _proof, _receiptHash);
        if (!success) {
            message = "mpt verification failed";
        } else {
            message = "success";
        }
    }

    function decodeHeader(bytes memory rlpBytes) external pure override returns (blockHeader memory bh) {
        RLPReader.RLPItem[] memory ls = rlpBytes.toRlpItem().toList();
        bh = blockHeader({
            parentHash: ls[0].toBytes(),
            coinbase: ls[1].toAddress(),
            root: ls[2].toBytes(),
            txHash: ls[3].toBytes(),
            receiptHash: ls[4].toBytes(),
            number: ls[6].toUint(),
            extraData: ls[10].toBytes(),
            bloom: ls[5].toBytes(),
            gasLimit: ls[7].toUint(),
            gasUsed: ls[8].toUint(),
            time: ls[9].toUint(),
            mixDigest: ls[11].toBytes(),
            nonce: ls[12].toBytes(),
            baseFee: ls[13].toUint()
        });
    }

    function encodeHeader(
        blockHeader memory _bh,
        bytes memory _deleteAggBytes,
        bytes memory _deleteSealAndAggBytes
    ) external view returns (bytes memory deleteAggHeaderBytes, bytes memory deleteSealAndAggHeaderBytes) {
        LibRLP.List memory list = LibRLP.l();
        LibRLP.p(list,_bh.parentHash);
        LibRLP.p(list,_bh.coinbase);
        LibRLP.p(list,_bh.root);
        LibRLP.p(list,_bh.txHash);
        LibRLP.p(list,_bh.receiptHash);
        LibRLP.p(list,_bh.bloom);
        LibRLP.p(list,_bh.number);
        LibRLP.p(list,_bh.gasLimit);
        LibRLP.p(list,_bh.gasUsed);
        LibRLP.p(list,_bh.time);
        LibRLP.p(list,_deleteAggBytes);
        LibRLP.p(list,_bh.mixDigest);
        LibRLP.p(list,_bh.nonce);
        LibRLP.p(list,_bh.baseFee);
        deleteAggHeaderBytes = LibRLP.encode(list);

        deleteSealAndAggHeaderBytes = _getDeleteSealAndAggHeaderBytes(_bh,_deleteSealAndAggBytes);
    }

    function _getDeleteSealAndAggHeaderBytes(blockHeader memory _bh,bytes memory _deleteSealAndAggBytes)
    internal
    view
    returns (bytes memory deleteSealAndAggHeaderBytes)
    {
        LibRLP.List memory list = LibRLP.l();
        LibRLP.p(list,_bh.parentHash);
        LibRLP.p(list,_bh.coinbase);
        LibRLP.p(list,_bh.root);
        LibRLP.p(list,_bh.txHash);
        LibRLP.p(list,_bh.receiptHash);
        LibRLP.p(list,_bh.bloom);
        LibRLP.p(list,_bh.number);
        LibRLP.p(list,_bh.gasLimit);
        LibRLP.p(list,_bh.gasUsed);
        LibRLP.p(list,_bh.time);
        LibRLP.p(list,_deleteSealAndAggBytes);
        LibRLP.p(list,_bh.mixDigest);
        LibRLP.p(list,_bh.nonce);
        LibRLP.p(list,_bh.baseFee);
        deleteSealAndAggHeaderBytes = LibRLP.encode(list);
    }

    function manageAgg(istanbulExtra memory ist)
    external
    view
    returns (bytes memory deleteAggBytes, bytes memory deleteSealAndAggBytes)
    {
        LibRLP.List memory list1 = LibRLP.l();
        LibRLP.List memory list2 = LibRLP.l();
        LibRLP.List memory list3 = LibRLP.l();

        for (uint256 i = 0; i < ist.validators.length; i++) {
            LibRLP.p(list1,ist.validators[i]);
        }
        for (uint256 i = 0; i < ist.addedPubKey.length; i++) {
            LibRLP.p(list2,ist.addedPubKey[i]);
        }
        for (uint256 i = 0; i < ist.addedG1PubKey.length; i++) {
            LibRLP.p(list3,ist.addedG1PubKey[i]);
        }

        LibRLP.List memory manageList = LibRLP.l();
        LibRLP.p(manageList,list1);
        LibRLP.p(manageList,list2);
        LibRLP.p(manageList,list3);
        LibRLP.p(manageList,ist.removeList);
        LibRLP.p(manageList,ist.seal);
        LibRLP.List memory list5 = LibRLP.l();
        LibRLP.p(list5,bytes(""));
        LibRLP.p(list5,bytes(""));
        LibRLP.p(list5,bytes(""));
        LibRLP.p(manageList,list5);
        LibRLP.List memory list6 = LibRLP.l();
        LibRLP.p(manageList,encodeAggregatedSeal(ist.parentAggregatedSeal.bitmap,ist.parentAggregatedSeal.signature,ist.parentAggregatedSeal.round));
        deleteAggBytes = LibRLP.encode(manageList);

        deleteSealAndAggBytes = getDeleteSealAndAggBytes(ist);

    }


    function getDeleteSealAndAggBytes(istanbulExtra memory ist) internal view returns(bytes memory deleteSealAndAggBytes){
        LibRLP.List memory list1 = LibRLP.l();
        LibRLP.List memory list2 = LibRLP.l();
        LibRLP.List memory list3 = LibRLP.l();

        for (uint256 i = 0; i < ist.validators.length; i++) {
            LibRLP.p(list1,ist.validators[i]);
        }
        for (uint256 i = 0; i < ist.addedPubKey.length; i++) {
            LibRLP.p(list2,ist.addedPubKey[i]);
        }
        for (uint256 i = 0; i < ist.addedG1PubKey.length; i++) {
            LibRLP.p(list3,ist.addedG1PubKey[i]);
        }

        LibRLP.List memory manageList = LibRLP.l();
        LibRLP.p(manageList,list1);
        LibRLP.p(manageList,list2);
        LibRLP.p(manageList,list3);
        LibRLP.p(manageList,ist.removeList);
        LibRLP.p(manageList,bytes(""));
        LibRLP.List memory list5 = LibRLP.l();
        LibRLP.p(list5,bytes(""));
        LibRLP.p(list5,bytes(""));
        LibRLP.p(list5,bytes(""));
        LibRLP.p(manageList,list5);
        LibRLP.p(manageList,encodeAggregatedSeal(ist.parentAggregatedSeal.bitmap,ist.parentAggregatedSeal.signature,ist.parentAggregatedSeal.round));
        deleteSealAndAggBytes = LibRLP.encode(manageList);
    }


    function decodeTxReceipt(bytes memory _receiptRlp) external pure override returns (bytes memory logHash) {
        // RLPReader.RLPItem[] memory ls = _receiptRlp.toRlpItem().toList();
        // logHash = RLPReader.toRlpBytes(ls[3]);

        return _receiptRlp.toRlpItem().safeGetItemByIndex(3).toRlpBytes();
    }

    function verifyHeader(
        address _coinbase,
        bytes memory _seal,
        bytes memory _headerWithoutSealAndAgg
    ) public pure override returns (bool ret, bytes32 headerHash) {
        headerHash = keccak256(abi.encodePacked(keccak256(abi.encodePacked(_headerWithoutSealAndAgg))));
        ret = verifySign(_seal, headerHash, _coinbase);
    }

    function getVerifyExpectedValueHash(
        uint256 _receiptType,
        bytes memory receiptRlp
    ) internal pure returns (bytes memory expectedValue) {
        if (_receiptType == 0) {
            return receiptRlp;
        } else {
            bytes memory tempType = abi.encode(_receiptType);
            bytes1 tip = tempType[31];
            return abi.encodePacked(tip, receiptRlp);
        }
    }

    function splitExtra(bytes memory extra) internal pure returns (bytes memory newExtra) {
        require(extra.length > 32, "invalid extra result type");
        newExtra = new bytes(extra.length - 32);
        uint256 n = 0;
        for (uint256 i = 32; i < extra.length; i++) {
            newExtra[n] = extra[i];
            n = n + 1;
        }
        return newExtra;
    }

    function encodeAggregatedSeal(
        uint256 bitmap,
        bytes memory signature,
        uint256 round
    ) internal pure returns (LibRLP.List memory list) {
        LibRLP.p(list,bitmap);
        LibRLP.p(list,signature);
        LibRLP.p(list,round);
    }

    function verifySign(
        bytes memory seal,
        bytes32 hash,
        address coinbase
    ) internal pure returns (bool) {
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
