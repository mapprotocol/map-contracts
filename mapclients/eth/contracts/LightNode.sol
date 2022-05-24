// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./lib/RLPReader.sol";
import "./lib/RLPEncode.sol";
import "./interface/ILightNode.sol";
import "./bls/WeightedMultiSig.sol";

interface IVerifyProof {
    function verifyTrieProof(bytes32 hash, bytes memory _expectedValue, bytes[] memory proofs,bytes memory _key)
    pure external returns (bool success);
}

contract LightNode is UUPSUpgradeable, Initializable, ILightNode, WeightedMultiSig{
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;


    uint256 public epochSize = 1000;
    address[] public validatorAddresss ;
    event validitorsSet(uint256 epoch);

    IVerifyProof verifyProof = IVerifyProof(0xdf713d32535126f3489431711be238DCA44DC808);

    constructor()  {}

    /** initialize  **********************************************************/
    function initialize(uint _threshold, address[]  memory _validatorAddresss, G1[] memory _pairKeys, uint[] memory _weights, uint _epoch, uint _epochSize)
    external initializer {
        epochSize = _epochSize;
        validatorAddresss = _validatorAddresss;
        setStateInternal(_threshold,_pairKeys,_weights,_epoch);
    }

    function verifyProofData(proveData memory _proveData, G2 memory aggPk) external view returns (bool success, string memory message) {
        (success, message) = getVerifyTrieProof(_proveData);
        if (!success) {
            message = "receipt mismatch";
        }
        //todo verify header
    }

    function updateBlockHeader(blockHeader memory bh, G2 memory aggPk) external {
        require(bh.number % epochSize == 0, "Header number is error");

        //todo verify
        //checkSig()
    }

    /** external function *********************************************************/


    function getStatus(bool _tag) public view returns (bytes memory PostStateOrStatus){
        PostStateOrStatus = abi.encode(_tag);
    }

    function _decodeExpectedValue(bytes memory rlpBytes)
    public
    pure
    returns (bytes memory output1, uint256 output2, bytes memory output3, Log[] memory logs){
        RLPReader.RLPItem[] memory i5ls = rlpBytes.toRlpItem().toList();
        // RLPReader.RLPItem[] memory i5ls = ls[5].toList(); //logs
        output1 = i5ls[0].toBytes();
        output2 = i5ls[1].toUint();
        output3 = i5ls[2].toBytes();
        //output4 = i5ls[3].toRlpItem().toList();
        RLPReader.RLPItem[] memory logInfo = i5ls[3].toList();

        uint256 num = logInfo.length;
        logs = new Log[](num);
        for (uint256 i = 0; i < num; i++) {
            RLPReader.RLPItem[] memory l = logInfo[i].toList();
            logs[i].addr = l[0].toAddress();

            RLPReader.RLPItem[] memory topicls = l[1].toList();
            uint256 n1 = topicls.length;
            logs[i].topics = new bytes[](n1);
            for (uint256 j = 0; j < n1; j++) {
                logs[i].topics[j] = topicls[j].toBytes();
            }

            logs[i].data = l[2].toBytes();
        }
    }


    function getVerifyExpectedValueHash(txLogs memory _txlogs) external pure returns (bytes memory output){
        bytes[] memory list = new bytes[](4);
        list[0] = RLPEncode.encodeBytes(_txlogs.PostStateOrStatus);
        list[1] = RLPEncode.encodeUint(_txlogs.CumulativeGasUsed);
        list[2] = RLPEncode.encodeBytes(_txlogs.Bloom);
        bytes[] memory listLog = new bytes[](_txlogs.logs.length);

        bytes[] memory loglist = new bytes[](3);

        for (uint256 j = 0; j < _txlogs.logs.length; j++) {
            loglist[0] = RLPEncode.encodeAddress(_txlogs.logs[j].addr);

            bytes[] memory loglist1 = new bytes[](_txlogs.logs[j].topics.length);
            for (uint256 i = 0; i < _txlogs.logs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes(_txlogs.logs[j].topics[i]);
            }

            loglist[1] = RLPEncode.encodeList(loglist1);

            loglist[2] = RLPEncode.encodeBytes(_txlogs.logs[j].data);

            //listLog[0] =  RLPEncode.encodeList(loglist);
            bytes memory logBytes = RLPEncode.encodeList(loglist);

            listLog[j] = RLPEncode.encodeBytes(logBytes);

            //list[3] = new bytes[](_txlogs.logs.length);

        }
        list[3] = RLPEncode.encodeList(listLog);
        output = RLPEncode.encodeList(list);
    }


    function getVerifyTrieProof(proveData memory _proveData) public view returns (
        bool success, string memory message){

        success = verifyProof.verifyTrieProof(bytes32(_proveData.header.receipHash), _proveData.prove.expectedValue,
            _proveData.prove.prove, _proveData.prove.keyIndex);
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
        receipHash : ls[4].toBytes(),
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
        //
        list[1] = RLPEncode.encodeAddress(bh.coinbase);
        //
        list[2] = RLPEncode.encodeBytes(bh.root);
        //
        list[3] = RLPEncode.encodeBytes(bh.txHash);
        //
        list[4] = RLPEncode.encodeBytes(bh.receipHash);
        //
        list[5] = RLPEncode.encodeBytes(bh.bloom);
        //
        list[6] = RLPEncode.encodeUint(bh.number);
        //
        list[7] = RLPEncode.encodeUint(bh.gasLimit);
        //;
        list[8] = RLPEncode.encodeUint(bh.gasUsed);
        //
        list[9] = RLPEncode.encodeUint(bh.time);
        //
        list[10] = RLPEncode.encodeBytes(bh.extraData);
        //
        list[11] = RLPEncode.encodeBytes(bh.mixDigest);
        //
        list[12] = RLPEncode.encodeBytes(bh.nonce);
        //
        list[13] = RLPEncode.encodeUint(bh.baseFee);
        //
        output = RLPEncode.encodeList(list);
    }

    function _decodeExtraData(bytes memory extraData)
    public
    pure
    returns (istanbulExtra memory ist){
        bytes memory decodeBytes = _splitExtra(extraData);
        RLPReader.RLPItem[] memory ls = decodeBytes.toRlpItem().toList();
        RLPReader.RLPItem memory item0 = ls[0];
        RLPReader.RLPItem memory item1 = ls[1];
        RLPReader.RLPItem memory item2 = ls[2];
        RLPReader.RLPItem memory item3 = ls[3];
        RLPReader.RLPItem memory item4 = ls[4];
        RLPReader.RLPItem memory item5 = ls[5];

        ist = istanbulExtra({
        removeList : item2.toUint(),
        seal : item3.toBytes(),
        aggregatedSeal : istanbulAggregatedSeal({
        round : item4.toList()[2].toUint(),
        signature : item4.toList()[1].toBytes(),
        bitmap : item4.toList()[0].toUint()
        }),
        parentAggregatedSeal : istanbulAggregatedSeal({
        round : item5.toList()[2].toUint(),
        signature : item5.toList()[1].toBytes(),
        bitmap : item5.toList()[0].toUint()
        }),
        validators : new address[](0),
        addedPubKey : new bytes[](0)
        });
        if (item0.len > 20) {
            uint256 num = item0.len / 20;
            ist.validators = new address[](num);
            ist.addedPubKey = new bytes[](num);
            for (uint256 i = 0; i < num; i++) {
                ist.validators[i] = item0.toList()[i].toAddress();
                ist.addedPubKey[i] = item1.toList()[i].toBytes();
            }
        }
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

        //the blockHash is the hash of the header without aggregated seal by validators.
        bytes memory blsMsg1 = _addsuffix(
            hash1,
            uint8(ist.aggregatedSeal.round)
        );
        if (bh.number % maxValidators == 0) {
            //ret = verifyAggregatedSeal(allkey[nowEpoch],ist.aggregatedSeal.signature,blsMsg1);
            //it need to update validators at first block of new epoch.
            // changeValidators(ist.removeList,ist.addedPubKey);
            removeList = ist.removeList;
            addedPubKey = ist.addedPubKey;
        } else {
            //ret = verifyAggregatedSeal(allkey[nowEpoch],ist.aggregatedSeal.signature,blsMsg1);
        }
        // emit log("verify msg of AggregatedSeal",blsMsg1);

        //the parent seal need to pks of last epoch to verify parent seal,if block number is the first block or the second block at new epoch.
        //because, the parent seal of the first block and the second block is signed by validitors of last epoch.
        //and it need to not verify, when the block number is less than 2, the block is no parent seal.
        bytes memory blsMsg2 = _addsuffix(
            hash1,
            uint8(ist.aggregatedSeal.round)
        );
        if (bh.number > 1) {
            if (
                (bh.number - 1) % maxValidators == 0 ||
                (bh.number) % maxValidators == 0
            ) {
                //ret = verifyAggregatedSeal(allkey[nowEpoch-1],ist.parentAggregatedSeal.signature,blsMsg2);
            } else {
                //ret = verifyAggregatedSeal(allkey[nowEpoch],ist.parentAggregatedSeal.signature,blsMsg2);
            }
        }
        // emit log("verify msg of ParentAggregatedSeal",blsMsg2);
    }


    function _deleteAgg(istanbulExtra memory ist, bytes memory extraDataPre)
    public
    pure
    returns (bytes memory newExtra){
        bytes[] memory list1 = new bytes[](ist.validators.length);
        bytes[] memory list2 = new bytes[](ist.addedPubKey.length);
        for (uint i = 0; i < ist.validators.length; i++) {
            list1[i] = RLPEncode.encodeAddress(ist.validators[i]);
            //
            list2[i] = RLPEncode.encodeBytes(ist.addedPubKey[i]);
            //
        }

        bytes[] memory list = new bytes[](6);
        list[0] = RLPEncode.encodeList(list1);
        //
        list[1] = RLPEncode.encodeList(list2);
        //
        list[2] = RLPEncode.encodeUint(ist.removeList);
        //
        list[3] = RLPEncode.encodeBytes(ist.seal);
        //
        list[4] = new bytes(4);
        list[4][0] = bytes1(uint8(195));
        list[4][1] = bytes1(uint8(128));
        list[4][2] = bytes1(uint8(128));
        list[4][3] = bytes1(uint8(128));
        list[5] = _encodeAggregatedSeal(ist.parentAggregatedSeal.bitmap, ist.parentAggregatedSeal.signature, ist.parentAggregatedSeal.round);
        bytes memory b = RLPEncode.encodeList(list);
        bytes memory output = new bytes(b.length + 32);
        for (uint i = 0; i < b.length + 32; i++) {
            if (i < 32) {
                output[i] = extraDataPre[i];
            } else {
                output[i] = b[i - 32];
            }
        }
        newExtra = output;
    }


    function _deleteSealAndAgg(istanbulExtra memory ist, bytes memory extraData)
    public
    pure
    returns (bytes memory newExtra){
        bytes[] memory list1 = new bytes[](ist.validators.length);
        bytes[] memory list2 = new bytes[](ist.addedPubKey.length);
        for (uint256 i = 0; i < ist.validators.length; i++) {
            list1[i] = RLPEncode.encodeAddress(ist.validators[i]);
            //
            list2[i] = RLPEncode.encodeBytes(ist.addedPubKey[i]);
            //
        }

        bytes[] memory list = new bytes[](6);
        list[0] = RLPEncode.encodeList(list1);
        //
        list[1] = RLPEncode.encodeList(list2);
        //
        list[2] = RLPEncode.encodeUint(ist.removeList);
        //
        list[3] = new bytes(1);
        list[3][0] = bytes1(uint8(128));
        //
        list[4] = new bytes(4);
        list[4][0] = bytes1(uint8(195));
        list[4][1] = bytes1(uint8(128));
        list[4][2] = bytes1(uint8(128));
        list[4][3] = bytes1(uint8(128));
        list[5] = _encodeAggregatedSeal(
            ist.parentAggregatedSeal.bitmap,
            ist.parentAggregatedSeal.signature,
            ist.parentAggregatedSeal.round
        );
        bytes memory b = RLPEncode.encodeList(list);
        bytes memory output = new bytes(b.length + 32);
        for (uint256 i = 0; i < b.length + 32; i++) {
            if (i < 32) {
                output[i] = extraData[i];
            } else {
                output[i] = b[i - 32];
            }
        }
        newExtra = output;
    }

    function _deleteSealAndAggTest(istanbulExtra memory ist, bytes memory extraData)
    public
    pure
    returns (bytes memory newExtra){
        bytes[] memory list1 = new bytes[](ist.validators.length);
        bytes[] memory list2 = new bytes[](ist.addedPubKey.length);
        for (uint256 i = 0; i < ist.validators.length; i++) {
            list1[i] = RLPEncode.encodeAddress(ist.validators[i]);
            //
            list2[i] = RLPEncode.encodeBytes(ist.addedPubKey[i]);
            //
        }

        bytes[] memory list = new bytes[](6);
        list[0] = RLPEncode.encodeList(list1);
        //
        list[1] = RLPEncode.encodeList(list2);
        //
        list[2] = RLPEncode.encodeUint(ist.removeList);
        //
        list[3] = new bytes(1);
        list[3][0] = bytes1(uint8(128));
        //
        list[4] = new bytes(4);
        list[4][0] = bytes1(uint8(195));
        list[4][1] = bytes1(uint8(128));
        list[4][2] = bytes1(uint8(128));
        list[4][3] = bytes1(uint8(128));
        list[5] = _encodeAggregatedSeal(
            ist.parentAggregatedSeal.bitmap,
            ist.parentAggregatedSeal.signature,
            ist.parentAggregatedSeal.round
        );
        bytes memory b = RLPEncode.encodeList(list);
        //        bytes memory output  = new bytes(b.length + 32);
        //        for (uint256 i = 0; i < b.length + 32; i++) {
        //            if (i < 32) {
        //                output[i] = extraData[i];
        //            } else {
        //                output[i] = b[i - 32];
        //            }
        //        }
        newExtra = b;
    }


    function _encodeAggregatedSeal(
        uint256 bitmap,
        bytes memory signature,
        uint256 round
    ) public pure returns (bytes memory output) {
        bytes memory output1 = RLPEncode.encodeUint(bitmap);
        //round
        bytes memory output2 = RLPEncode.encodeBytes(signature);
        //signature
        bytes memory output3 = RLPEncode.encodeUint(round);
        //bitmap

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
        //round
        bytes memory output2 = RLPEncode.encodeBytes(signature);
        //signature
        bytes memory output3 = RLPEncode.encodeAddress(round);
        //bitmap

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
        //Signature storaged in extraData sub 27 after proposer signed.
        //So signature need to add 27 when verify it.
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

    //suffix's rule is hash + round + commitMsg(the value is 2 usually);
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
