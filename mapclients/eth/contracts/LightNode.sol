// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./lib/RLPReader.sol";
import "./lib/RLPEncode.sol";
import "./interface/ILightNode.sol";
import "./bls/WeightedMultiSig.sol";
import "./lib/MPT.sol";

interface IVerifyProof {
    function verifyTrieProof(bytes32 hash, bytes memory _expectedValue, bytes[] memory proofs,bytes memory _key)
    pure external returns (bool success);
}

contract LightNode is UUPSUpgradeable, Initializable, ILightNode{
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;


    uint256 public epochSize = 1000;
    address[] public validatorAddresss ;
    event validitorsSet(uint256 epoch);

    WeightedMultiSig weightedMultisig;

    constructor()  {}

    /** initialize  **********************************************************/
    function initialize(uint _threshold, address[]  memory _validatorAddresss, G1[] memory _pairKeys,
        uint[] memory _weights, uint _epoch, uint _epochSize)
    external initializer {
        epochSize = _epochSize;
        validatorAddresss = _validatorAddresss;
        weightedMultisig.setStateInternal(_threshold,_pairKeys,_weights,_epoch);
    }

    function verifyProofData(receiptProof memory _receiptProof) external view returns (bool success, string memory message){
        (success, message) = getVerifyTrieProof(_receiptProof);
        if (!success) {
            message = "receipt mismatch";
        }
        //todo verify header
    }

    function updateBlockHeader(blockHeader memory bh, G2 memory aggPk) external {
        require(bh.number % epochSize == 0, "Header number is error");

        //todo verify
        //weightedMultisig.checkSig()
    }


    function getStatus(bool _tag) public view returns (bytes memory PostStateOrStatus){
        PostStateOrStatus = abi.encode(_tag);
    }


    function getVerifyTrieProof(receiptProof memory _receiptProof) public view returns (
        bool success, string memory message){
        MPT.MerkleProof memory mProof ;
        //todo set proof
        success = MPT.verifyTrieProof(mProof);
        if (!success) {
            message = "receipt mismatch";
        } else {
            message = "success";
        }
    }


    function _bytesSlice32(bytes memory data, uint256 offset)
    public
    pure
    returns (uint256 slice){
        bytes memory tmp = new bytes(32);
        for (uint256 i = 0; i < 32; i++) {
            tmp[i] = data[offset + i];
        }
        slice = uint256(bytes32(tmp));
    }

    function _decodeHeader(bytes memory rlpBytes)
    public
    pure
    returns (blockHeader memory bh){
        RLPReader.RLPItem[] memory ls = rlpBytes.toRlpItem().toList();
        bh = blockHeader({
        parentHash : ls[0].toBytes(),
        coinbase : ls[1].toAddress(),
        root : ls[2].toBytes(),
        txHash : ls[3].toBytes(),
        receiptHash : ls[4].toBytes(),
        number : ls[6].toUint(),
        extraData : ls[10].toBytes(),
        bloom : ls[5].toBytes(),
        gasLimit : ls[7].toUint(),
        gasUsed : ls[8].toUint(),
        time : ls[9].toUint(),
        mixDigest : ls[11].toBytes(),
        nonce : ls[12].toBytes(),
        baseFee : ls[13].toUint()
        });
    }


    function _encodeHeader(blockHeader memory bh)
    public
    pure
    returns (bytes memory output){
        bytes[] memory list = new bytes[](14);
        list[0] = RLPEncode.encodeBytes(bh.parentHash);

        list[1] = RLPEncode.encodeAddress(bh.coinbase);

        list[2] = RLPEncode.encodeBytes(bh.root);

        list[3] = RLPEncode.encodeBytes(bh.txHash);

        list[4] = RLPEncode.encodeBytes(bh.receiptHash);

        list[5] = RLPEncode.encodeBytes(bh.bloom);

        list[6] = RLPEncode.encodeUint(bh.number);

        list[7] = RLPEncode.encodeUint(bh.gasLimit);

        list[8] = RLPEncode.encodeUint(bh.gasUsed);

        list[9] = RLPEncode.encodeUint(bh.time);

        list[10] = RLPEncode.encodeBytes(bh.extraData);

        list[11] = RLPEncode.encodeBytes(bh.mixDigest);

        list[12] = RLPEncode.encodeBytes(bh.nonce);

        list[13] = RLPEncode.encodeUint(bh.baseFee);

        output = RLPEncode.encodeList(list);
    }

    function _decodeExtraData(bytes memory extraData)
    public
    pure
    returns (istanbulExtra memory ist){
        bytes memory decodeBytes = _splitExtra(extraData);
        RLPReader.RLPItem[] memory ls = decodeBytes.toRlpItem().toList();
        RLPReader.RLPItem[] memory item0 = ls[0].toList();
        RLPReader.RLPItem[] memory item1 = ls[1].toList();
        RLPReader.RLPItem[] memory item2 = ls[2].toList();
        RLPReader.RLPItem memory item3 = ls[3];
        RLPReader.RLPItem memory item4 = ls[4];
        RLPReader.RLPItem memory item5 = ls[5];
        RLPReader.RLPItem memory item6 = ls[6];

        address[] memory validatorTemp = new address[](item0.length);
        bytes[] memory addedPubKeyTemp = new bytes[](item1.length);
        bytes[] memory addedG1PubKeyTemp = new bytes[](item2.length);

        if (item0.length >0 ){
            for (uint256 i = 0; i < item0.length; i++) {
                validatorTemp[i] = item0[i].toAddress();
            }
        }

        if (item1.length >0 ){
            for (uint256 j = 0; j < item1.length; j++) {
                addedPubKeyTemp[j] = item1[j].toBytes();
            }
        }

        if (item2.length >0 ){
            for (uint256 k = 0; k < item2.length; k++) {
                addedG1PubKeyTemp[k] = item2[k].toBytes();
            }
        }

        ist = istanbulExtra({
        validators: validatorTemp,
        addedPubKey: addedPubKeyTemp,
        addedG1PubKey: addedG1PubKeyTemp,
        removeList: item3.toUint(),
        seal: item4.toBytes(),
        aggregatedSeal: istanbulAggregatedSeal({
        bitmap: item5.toList()[0].toUint(),
        signature: item5.toList()[1].toBytes(),
        round: item5.toList()[2].toUint()
        }),
        parentAggregatedSeal: istanbulAggregatedSeal({
        bitmap: item6.toList()[0].toUint(),
        signature: item6.toList()[1].toBytes(),
        round: item6.toList()[2].toUint()
        })
        });

    }


    function _splitExtra(bytes memory extra)
    public
    pure
    returns (bytes memory newExtra){
        newExtra = new bytes(extra.length - 32);
        uint256 n = 0;
        for (uint256 i = 32; i < extra.length; i++) {
            newExtra[n] = extra[i];
            n = n + 1;
        }
        return newExtra;
    }

    function splitExtra(bytes memory extra)
    public
    pure
    returns (bytes memory newExtra){
        newExtra = new bytes(32);
        uint m = 0;
        for (uint i = 0; i < 32; i++) {
            newExtra[m] = extra[i];
            m = m + 1;
        }
        return newExtra;
    }

    function _verifyHeader(bytes memory rlpHeader)
    public
    view
    returns (bool ret, uint256 removeList, bytes[] memory addedPubKey){
        blockHeader memory bh = _decodeHeader(rlpHeader);
        istanbulExtra memory ist = _decodeExtraData(bh.extraData);
        bytes memory extraDataPre = splitExtra(bh.extraData);
        bh.extraData = _deleteAgg(ist, extraDataPre);
        bytes memory headerWithoutAgg = _encodeHeader(bh);
        bytes32 hash1 = keccak256(abi.encodePacked(headerWithoutAgg));
        bh.extraData = _deleteSealAndAgg(ist, bh.extraData);
        bytes memory headerWithoutSealAndAgg = _encodeHeader(bh);
        bytes32 hash2 = keccak256(abi.encodePacked(headerWithoutSealAndAgg));

        ret = _verifySign(
            ist.seal,
            keccak256(abi.encodePacked(hash2)),
            bh.coinbase
        );
        if (ret == false) {
            revert("verifyEscaSign fail");
        }
    }

    function _deleteAgg(istanbulExtra memory ist,bytes memory extraDataPre)
    public
    pure
    returns (bytes memory newExtra){
        bytes[] memory list1 = new bytes[](ist.validators.length);
        bytes[] memory list2 = new bytes[](ist.addedPubKey.length);
        bytes[] memory list3 = new bytes[](ist.addedG1PubKey.length);
        for (uint256 i = 0; i < ist.validators.length; i++) {
            list1[i] = RLPEncode.encodeAddress(ist.validators[i]); //
            list2[i] = RLPEncode.encodeBytes(ist.addedPubKey[i]); //
            list3[i] = RLPEncode.encodeBytes(ist.addedG1PubKey[i]);
        }

        bytes[] memory list = new bytes[](7);
        list[0] = RLPEncode.encodeList(list1);//
        list[1] = RLPEncode.encodeList(list2);//
        list[2] = RLPEncode.encodeList(list3);
        list[3] = RLPEncode.encodeUint(ist.removeList);//
        list[4] = RLPEncode.encodeBytes(ist.seal);//
        list[5] = new bytes(4);
        list[5][0] = bytes1(uint8(195));
        list[5][1] = bytes1(uint8(128));
        list[5][2] = bytes1(uint8(128));
        list[5][3] = bytes1(uint8(128));
        list[6] = _encodeAggregatedSeal(ist.parentAggregatedSeal.bitmap,ist.parentAggregatedSeal.signature,ist.parentAggregatedSeal.round);
        bytes memory b = RLPEncode.encodeList(list);
        bytes memory output = new bytes(b.length+32);
        for (uint i=0;i<b.length+32;i++){
            if (i<32){
                output[i] = extraDataPre[i];
            }else{
                output[i] = b[i-32];
            }
        }
        newExtra = output;
    }


    function _deleteSealAndAgg(istanbulExtra memory ist,bytes memory rlpHeader)
    public
    pure
    returns (bytes memory newExtra){
        bytes[] memory list1 = new bytes[](ist.validators.length);
        bytes[] memory list2 = new bytes[](ist.addedPubKey.length);
        bytes[] memory list3 = new bytes[](ist.addedG1PubKey.length);
        for (uint256 i = 0; i < ist.validators.length; i++) {
            list1[i] = RLPEncode.encodeAddress(ist.validators[i]); //
            list2[i] = RLPEncode.encodeBytes(ist.addedPubKey[i]); //
            list3[i] = RLPEncode.encodeBytes(ist.addedG1PubKey[i]);
        }

        bytes[] memory list = new bytes[](7);
        list[0] = RLPEncode.encodeList(list1); //
        list[1] = RLPEncode.encodeList(list2); //
        list[2] = RLPEncode.encodeList(list3); //
        list[3] = RLPEncode.encodeUint(ist.removeList); //
        list[4] = new bytes(1);
        list[4][0] = bytes1(uint8(128)); //
        list[5] = new bytes(4);
        list[5][0] = bytes1(uint8(195));
        list[5][1] = bytes1(uint8(128));
        list[5][2] = bytes1(uint8(128));
        list[5][3] = bytes1(uint8(128));
        list[6] = _encodeAggregatedSeal(
            ist.parentAggregatedSeal.bitmap,
            ist.parentAggregatedSeal.signature,
            ist.parentAggregatedSeal.round
        );
        bytes memory b = RLPEncode.encodeList(list);


        newExtra = abi.encodePacked(bytes32(rlpHeader),b);
    }


    function _encodeAggregatedSeal(
        uint256 bitmap,
        bytes memory signature,
        uint256 round
    ) public pure returns (bytes memory output) {
        bytes memory output1 = RLPEncode.encodeUint(bitmap);

        bytes memory output2 = RLPEncode.encodeBytes(signature);

        bytes memory output3 = RLPEncode.encodeUint(round);


        bytes[] memory list = new bytes[](3);
        list[0] = output1;
        list[1] = output2;
        list[2] = output3;
        output = RLPEncode.encodeList(list);
    }

    function _encodeSealHash(
        uint256 bitmap,
        bytes memory signature,
        address round
    ) public pure returns (bytes memory output) {
        bytes memory output1 = RLPEncode.encodeUint(bitmap);

        bytes memory output2 = RLPEncode.encodeBytes(signature);

        bytes memory output3 = RLPEncode.encodeAddress(round);


        bytes[] memory list = new bytes[](3);
        list[0] = output1;
        list[1] = output2;
        list[2] = output3;
        output = RLPEncode.encodeList(list);
    }

    function _verifySign(
        bytes memory seal,
        bytes32 hash,
        address coinbase
    ) public pure returns (bool) {

        (bytes32 r, bytes32 s, uint8 v) = splitSignature(seal);
        v = v + 27;
        return coinbase == ecrecover(hash, v, r, s);
    }

    function splitSignature(bytes memory sig)
    public
    pure
    returns
    (bytes32 r, bytes32 s, uint8 v){
        require(sig.length == 65, "invalid signature length");
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
    }

    function _addsuffix(bytes32 hash, uint8 round)
    public
    pure
    returns (bytes memory){
        bytes memory result = new bytes(34);
        for (uint256 i = 0; i < 32; i++) {
            result[i] = hash[i];
        }
        result[32] = bytes1(round);
        result[33] = bytes1(uint8(2));
        return result;
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }
}
