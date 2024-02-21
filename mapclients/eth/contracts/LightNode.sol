// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/ILightNode.sol";
import "./bls/BlsCode.sol";
import "./bls/BGLS.sol";
import "./interface/IVerifyTool.sol";

contract LightNode is UUPSUpgradeable, Initializable, ILightNode, BGLS {

    uint256 public maxValidators;
    uint256 public epochSize;
    uint256 public headerHeight;
    address[] public validatorAddress;
    validator[] public validators;
    IVerifyTool public verifyTool;
    BlsCode blsCode;
    address private _pendingAdmin;
    uint256 public startHeight;

    struct validator {
        G1[] pairKeys; // <-- 100 validators, pubkey G2,   (s, s * g2)   s * g1
        uint[] weights; // voting power
        uint256 threshold; // bft, > 2/3,  if  \sum weights = 100, threshold = 67
        uint256 epoch;
    }

    validator verifier;

    mapping(uint256 => bytes32) private cachedReceiptRoot;

    event mapInitializeValidators(uint256 _threshold, G1[] _pairKeys, uint[] _weights, uint256 epoch);
    event MapUpdateValidators(G1[] _pairKeysAdd, uint[] _weights, uint256 epoch, bytes bits);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event AdminTransferred(address indexed previous, address indexed newAdmin);
    event NewVerifyTool(address newVerifyTool);

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "Lightnode only admin");
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor()  {}

    /** initialize  **********************************************************/
    function initialize(
        uint _threshold,
        address[]  memory _validatorAddress,
        G1[] memory _pairKeys,
        uint[] memory _weights,
        uint _epoch,
        uint _epochSize,
        address _verifyTool)
    external
    override
    initializer {
        require(_epoch > 1, "Error initializing epco");
        _changeAdmin(tx.origin);
        maxValidators = 1728000 / _epochSize;
        headerHeight = (_epoch - 1) * _epochSize;
        startHeight = headerHeight;
        epochSize = _epochSize;
        validatorAddress = _validatorAddress;
        for(uint256 i = 0; i < maxValidators ; i ++){
            validators.push(verifier);
        }
        setStateInternal(_threshold, _pairKeys, _weights, _epoch);
        verifyTool = IVerifyTool(_verifyTool);
        blsCode = new BlsCode();
        g1 = G1(1, 2);
        g2 = G2(
            0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
            0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
            0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
            0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b
        );
        emit mapInitializeValidators(_threshold, _pairKeys, _weights, _epoch);
    }

    function getValidator(uint id)
    public
    view
    returns (G1[] memory){
        return validators[id].pairKeys;
    }

    function getValiditors()
    public
    view
    returns (uint256){
        return maxValidators;
    }

    function getBytes(receiptProof memory _receiptProof) public pure returns (bytes memory){
        return abi.encode(_receiptProof);
    }

    function setVerifyTool(address _verifyTool) external onlyOwner {
        verifyTool = IVerifyTool(_verifyTool);
        emit NewVerifyTool(_verifyTool);
    }

    function verifyProofData(bytes memory _receiptProofBytes)
    external
    view
    override
    returns (bool success, string memory message, bytes memory logsHash) {
        receiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (receiptProof));

        return _verifyProofData(_receiptProof);

    }

    function getData(bytes memory _receiptProofBytes)
    external
    view
    returns (receiptProof memory) {
        receiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (receiptProof));

        return _receiptProof;

    }

    function verifyProofDataWithCache(bytes memory _receiptProofBytes)
    external
    override
    returns (bool success, string memory message,bytes memory logsHash){
        receiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (receiptProof));
        logsHash = verifyTool.decodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
        if(cachedReceiptRoot[_receiptProof.header.number] != bytes32("")){
            (success, message) = verifyTool.getVerifyTrieProof(
                cachedReceiptRoot[_receiptProof.header.number],
                _receiptProof.keyIndex,
                _receiptProof.proof,
                _receiptProof.txReceiptRlp.receiptRlp,
                _receiptProof.txReceiptRlp.receiptType
            );
            if (!success) {
                message = "Mpt verification failed";
                return (success, message, logsHash);
            }
        }else {
            (success,message,logsHash) = _verifyProofData(_receiptProof);
            if(success){
                cachedReceiptRoot[_receiptProof.header.number] = bytes32(_receiptProof.header.receiptHash);
            }
        }
    }

    function _verifyProofData(receiptProof memory _receiptProof)
    internal
    view
    returns(bool success, string memory message, bytes memory logsHash)
    {
        (uint min, uint max) = verifiableHeaderRange();
        uint height = _receiptProof.header.number;
        if (height <= min || height >= max) {
            message = "Out of verify range";
            return (false, message, logsHash);
        }

        logsHash = verifyTool.decodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
        (success, message) = verifyTool.getVerifyTrieProof(
            bytes32(_receiptProof.header.receiptHash),
            _receiptProof.keyIndex,
            _receiptProof.proof,
            _receiptProof.txReceiptRlp.receiptRlp,
            _receiptProof.txReceiptRlp.receiptType
        );
        if (!success) {
            message = "Mpt verification failed";
            return (success, message, logsHash);
        }
        success = verifyHeaderSig(_receiptProof.header, _receiptProof.ist, _receiptProof.aggPk);
        if (!success) {
            message = "VerifyHeaderSig fail";
            return (success, message, logsHash);
        }
        return (success, message, logsHash);
    }

    function updateBlockHeader(blockHeader memory bh, istanbulExtra memory ist, G2 memory aggPk)
    external
    override
    {
        require(bh.number % epochSize == 0, "Header number is error");
        require(bh.number - epochSize == headerHeight, "Header is have");
        headerHeight = bh.number;
        if (startHeight == 0) {
            startHeight = headerHeight - epochSize;
        }
        bool success = verifyHeaderSig(bh, ist, aggPk);
        require(success, "CheckSig error");
        uint256 len = ist.addedG1PubKey.length;
        G1[] memory _pairKeysAdd = new G1[](len);
        uint256[] memory _weights = new uint256[](len);
        if (len > 0) {
            for (uint256 i = 0; i < len; i++) {
                _weights[i] = 1;
                _pairKeysAdd[i] = blsCode.decodeG1(ist.addedG1PubKey[i]);
            }
        }
        bytes memory bits = abi.encodePacked(ist.removeList);
        uint256 epoch = getEpochNumber(bh.number) + 1;
        updateValidators(_pairKeysAdd, _weights, epoch, bits);
        emit UpdateBlockHeader(msg.sender, bh.number);
        emit MapUpdateValidators(_pairKeysAdd, _weights, epoch, bits);
    }

    function verifyHeaderSig(blockHeader memory _bh, istanbulExtra memory ist, G2 memory _aggPk)
    internal
    view
    returns (bool success){
        bytes32 extraDataPre = bytes32(_bh.extraData);
        (bytes memory deleteAggBytes,
        bytes memory deleteSealAndAggBytes) = verifyTool.manageAgg(ist);
        deleteAggBytes = abi.encodePacked(extraDataPre, deleteAggBytes);
        deleteSealAndAggBytes = abi.encodePacked(extraDataPre, deleteSealAndAggBytes);

        (bytes memory deleteAggHeaderBytes,bytes memory deleteSealAndAggHeaderBytes) =
        verifyTool.encodeHeader(_bh, deleteAggBytes, deleteSealAndAggBytes);

        (success,) = verifyTool.verifyHeader(_bh.coinbase, ist.seal, deleteSealAndAggHeaderBytes);
        if (!success) return success;
        success = checkSig(_bh, ist, _aggPk, deleteAggHeaderBytes);
        return success;
    }


    function verifiableHeaderRange() public view override returns (uint256, uint256){
        uint start;
        if(headerHeight > maxValidators * epochSize){
            start = headerHeight - (maxValidators * epochSize);
        }

        if (startHeight > 0 && startHeight > start) {
            start = startHeight;
        }
        return (start, headerHeight + epochSize);
    }

    function checkSig(blockHeader memory _bh, istanbulExtra memory _ist, G2 memory _aggPk, bytes memory _headerWithoutAgg)
    internal
    view
    returns (bool){
        uint256 epoch = getEpochNumber(_bh.number);
        bytes memory message = getPrepareCommittedSeal(_headerWithoutAgg, _ist.aggregatedSeal.round);
        bytes memory bits = abi.encodePacked(_ist.aggregatedSeal.bitmap);
        G1 memory sig = blsCode.decodeG1(_ist.aggregatedSeal.signature);
        return checkSigTag(bits, message, sig, _aggPk, epoch);
    }

    function setStateInternal(uint256 _threshold, G1[] memory _pairKeys, uint[] memory _weights, uint256 epoch)
    internal
    {
        require(_pairKeys.length == _weights.length, 'Mismatch arg');
        uint256 id = getValidatorsId(epoch);
        validator storage v = validators[id];

        for (uint256 i = 0; i < _pairKeys.length; i++) {
            v.pairKeys.push(
                G1({
            x : _pairKeys[i].x,
            y : _pairKeys[i].y
            }));
            v.weights.push(_weights[i]);
        }

        v.threshold = _threshold;
        v.epoch = epoch;
    }

    function updateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint256 epoch, bytes memory bits)
    internal
    {
        uint256 idPre = getValidatorsIdPrev(epoch);
        validator memory vPre = validators[idPre];
        uint256 id = getValidatorsId(epoch);
        validator storage v = validators[id];
        v.epoch = epoch;
        uint _weight = 0;
        if (v.pairKeys.length > 0) {
            delete (v.weights);
            delete (v.pairKeys);
        }

        for (uint256 i = 0; i < vPre.pairKeys.length; i++) {
            if (!chkBit(bits, i)) {
                v.pairKeys.push(
                    G1({
                x : vPre.pairKeys[i].x,
                y : vPre.pairKeys[i].y
                }));
                v.weights.push(vPre.weights[i]);
                _weight = _weight + vPre.weights[i];
            }
        }

        if (_pairKeysAdd.length > 0) {
            for (uint256 i = 0; i < _pairKeysAdd.length; i++) {
                v.pairKeys.push(
                    G1({
                x : _pairKeysAdd[i].x,
                y : _pairKeysAdd[i].y
                }));
                v.weights.push(_weights[i]);
                _weight = _weight + _weights[i];
            }
        }
        v.threshold = _weight - _weight / 3;
    }


    function getValidatorsId(uint256 epoch)
    internal
    view
    returns (uint){
        return epoch % maxValidators;
    }

    function getValidatorsIdPrev(uint256 epoch)
    internal
    view
    returns (uint){
        uint256 id = getValidatorsId(epoch);
        if (id == 0) {
            return maxValidators - 1;
        } else {
            return id - 1;
        }
    }

    function isQuorum(bytes memory bits, uint[] memory weights, uint256 threshold)
    internal
    pure
    returns (bool) {
        uint256 weight = 0;
        for (uint256 i = 0; i < weights.length; i++) {
            if (chkBit(bits, i)) weight += weights[i];
        }
        return weight >= threshold;
    }

    function checkAggPk(bytes memory bits, G2 memory aggPk, G1[] memory pairKeys)
    internal
    view
    returns (bool) {
        return pairingCheck(sumPoints(pairKeys, bits), g2, g1, aggPk);
    }

    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    //
    function checkSigTag(bytes memory bits, bytes memory message, G1 memory sig, G2 memory aggPk, uint256 epoch)
    internal
    view
    returns (bool) {
        uint256 id = getValidatorsId(epoch);
        validator memory v = validators[id];
        return isQuorum(bits, v.weights, v.threshold)
        && checkAggPk(bits, aggPk, v.pairKeys)
        && checkSignature(message, sig, aggPk);
    }

    function getPrepareCommittedSeal(bytes memory _headerWithoutAgg, uint256 _round)
    internal
    pure
    returns (bytes memory result){
        bytes32 hash = keccak256(_headerWithoutAgg);
        if (_round == 0) {
            result = abi.encodePacked(hash, uint8(2));
        } else {
            result = abi.encodePacked(hash, getLengthInBytes(_round), uint8(2));
        }
    }


    function getLengthInBytes(uint256 num)
    internal
    pure
    returns (bytes memory){
        require(num < 2 ** 24, "Num is too large");
        bytes memory result;
        if (num < 256) {
            result = abi.encodePacked(uint8(num));
        } else if (num < 65536) {
            result = abi.encodePacked(uint16(num));
        } else {
            result = abi.encodePacked(uint24(num));
        }
        return result;
    }


    function getEpochNumber(uint256 blockNumber)
    internal
    view
    returns (uint256){
        if (blockNumber % epochSize == 0) {
            return blockNumber / epochSize;
        }
        return blockNumber / epochSize + 1;
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address)
    internal
    view
    override {
        require(msg.sender == _getAdmin(), "LightNode only Admin can upgrade");
    }

    function changeAdmin() public {
        require(_pendingAdmin == msg.sender, "Only pendingAdmin");
        emit AdminTransferred(_getAdmin(), _pendingAdmin);
        _changeAdmin(_pendingAdmin);
        _pendingAdmin = address(0);
    }


    function pendingAdmin() external view returns (address){
        return _pendingAdmin;
    }

    function setPendingAdmin(address pendingAdmin_) public onlyOwner {
        require(pendingAdmin_ != address(0), "Ownable pendingAdmin is the zero address");
        emit ChangePendingAdmin(_pendingAdmin, pendingAdmin_);
        _pendingAdmin = pendingAdmin_;
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }

}