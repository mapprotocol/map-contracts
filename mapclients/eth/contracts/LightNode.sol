// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/ILightNode.sol";
import "./bls/BGLS.sol";
import "./interface/IVerifyTool.sol";
import "./interface/ILightNodePoint.sol";

contract LightNode is UUPSUpgradeable, Initializable, ILightNode {
    address private _pendingAdmin;

    uint256 public maxEpochs; // max epoch number
    uint256 public epochSize; // every epoch block number

    uint256 public startHeight;
    uint256 public headerHeight; // the last header
    // address[] public validatorAddress;
    Epoch[] public epochs;
    IVerifyTool public verifyTool;

    mapping(uint256 => bytes32) private cachedReceiptRoot;

    // BlsCode blsCode;

    Epoch verifier;

    struct TxReceiptRlp {
        uint256 receiptType;
        bytes receiptRlp;
    }

    struct ReceiptProof {
        ILightNodePoint.blockHeader header;
        ILightNodePoint.istanbulExtra ist;
        BGLS.G2 aggPk;
        TxReceiptRlp txReceiptRlp;
        bytes keyIndex;
        bytes[] proof;
    }

    struct Epoch {
        BGLS.G1[] pairKeys; // <-- 100 validators, pubkey G2,   (s, s * g2)   s * g1
        uint256[] weights; // voting power
        uint256 threshold; // bft, > 2/3,  if  \sum weights = 100, threshold = 67
        uint256 epoch;
    }

    event MapInitializeValidators(uint256 _threshold, BGLS.G1[] _pairKeys, uint256[] _weights, uint256 epoch);
    event MapUpdateValidators(BGLS.G1[] _pairKeysAdd, uint256[] _weights, uint256 epoch, bytes bits);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event AdminTransferred(address indexed previous, address indexed newAdmin);
    event NewVerifyTool(address newVerifyTool);

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "Lightnode only admin");
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {}

    /** initialize  **********************************************************/
    function initialize(
        uint256 _threshold,
        address[] memory _validatorAddress,
        BGLS.G1[] memory _pairKeys,
        uint256[] memory _weights,
        uint256 _epoch,
        uint256 _epochSize,
        address _verifyTool
    ) external initializer {
        require(_epoch > 1, "Error initializing epoch");
        _changeAdmin(tx.origin);
        maxEpochs = 1728000 / _epochSize;
        headerHeight = (_epoch - 1) * _epochSize;
        startHeight = headerHeight;
        epochSize = _epochSize;
        // validatorAddress = _validatorAddress;
        for (uint256 i = 0; i < maxEpochs; i++) {
            epochs.push(verifier);
        }
        setStateInternal(_threshold, _pairKeys, _weights, _epoch);
        verifyTool = IVerifyTool(_verifyTool);

        emit MapInitializeValidators(_threshold, _pairKeys, _weights, _epoch);
    }


    function updateBlockHeader(
        ILightNodePoint.blockHeader memory bh,
        ILightNodePoint.istanbulExtra memory ist,
        BGLS.G2 memory aggPk
    ) external {
        require(bh.number % epochSize == 0, "Header number is error");
        require(bh.number - epochSize == headerHeight, "Header is have");
        headerHeight = bh.number;
        if (startHeight == 0) {
            startHeight = headerHeight - epochSize;
        }
        bool success = _verifyHeaderSig(bh, ist, aggPk);
        require(success, "CheckSig error");

        _updateValidators(bh.number, ist);

        emit UpdateBlockHeader(msg.sender, bh.number);
    }


    function verifyProofDataWithCache(
        bytes memory _receiptProofBytes
    ) external override returns (bool success, string memory message, bytes memory logsHash) {
        ReceiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProof));

        if (cachedReceiptRoot[_receiptProof.header.number] != bytes32("")) {

            return _verifyMptProof(cachedReceiptRoot[_receiptProof.header.number], _receiptProof);
        } else {
            (success, message, logsHash) = _verifyProofData(_receiptProof);
            if (success) {
                cachedReceiptRoot[_receiptProof.header.number] = bytes32(_receiptProof.header.receiptHash);
            }
        }
    }


    /** view *********************************************************/

    function verifyProofData(
        bytes memory _receiptProofBytes
    ) external view override returns (bool success, string memory message, bytes memory logsHash) {
        ReceiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProof));

        return _verifyProofData(_receiptProof);
    }

    function getData(bytes memory _receiptProofBytes) external view returns (ReceiptProof memory) {
        ReceiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProof));

        return _receiptProof;
    }

    function getValidators(uint256 id) public view returns (BGLS.G1[] memory) {
        return epochs[id].pairKeys;
    }

    function getBytes(ReceiptProof memory _receiptProof) public pure returns (bytes memory) {
        return abi.encode(_receiptProof);
    }

    function setVerifyTool(address _verifyTool) external onlyOwner {
        verifyTool = IVerifyTool(_verifyTool);
        emit NewVerifyTool(_verifyTool);
    }

    function verifiableHeaderRange() public view override returns (uint256, uint256) {
        uint256 start;
        if (headerHeight > maxEpochs * epochSize) {
            start = headerHeight - (maxEpochs * epochSize);
        }

        if (startHeight > 0 && startHeight > start) {
            start = startHeight;
        }
        return (start, headerHeight + epochSize);
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        (uint256 start, uint256 end) = verifiableHeaderRange();
        return start <= _blockHeight && _blockHeight <= end;
    }

    function nodeType() external view override returns (uint256) {
        return 1;
    }

    function notifyLightClient(bytes memory _data) external override {
        emit NotifySend(msg.sender, block.number, _data);
    }


    /** internal *********************************************************/

    function setStateInternal(
        uint256 _threshold,
        BGLS.G1[] memory _pairKeys,
        uint256[] memory _weights,
        uint256 epoch
    ) internal {
        require(_pairKeys.length == _weights.length, "Mismatch arg");
        uint256 id = _getEpochId(epoch);
        Epoch storage v = epochs[id];

        for (uint256 i = 0; i < _pairKeys.length; i++) {
            v.pairKeys.push(BGLS.G1({x: _pairKeys[i].x, y: _pairKeys[i].y}));
            v.weights.push(_weights[i]);
        }

        v.threshold = _threshold;
        v.epoch = epoch;
    }


    function _updateValidators(uint256 _blockNumber, ILightNodePoint.istanbulExtra memory ist) internal {
        bytes memory bits = abi.encodePacked(ist.removeList);

        uint256 epoch = _getEpochByNumber(_blockNumber) + 1;
        uint256 idPre = _getPreEpochId(epoch);
        Epoch memory vPre = epochs[idPre];
        uint256 id = _getEpochId(epoch);
        Epoch storage v = epochs[id];
        v.epoch = epoch;

        if (v.pairKeys.length > 0) {
            delete (v.weights);
            delete (v.pairKeys);
        }

        uint256 _weight = 0;
        for (uint256 i = 0; i < vPre.pairKeys.length; i++) {
            if (!BGLS.chkBit(bits, i)) {
                v.pairKeys.push(BGLS.G1({x: vPre.pairKeys[i].x, y: vPre.pairKeys[i].y}));
                v.weights.push(vPre.weights[i]);
                _weight = _weight + vPre.weights[i];
            }
        }

        uint256 len = ist.addedG1PubKey.length;
        BGLS.G1[] memory _pairKeysAdd = new BGLS.G1[](len);
        uint256[] memory _weights = new uint256[](len);
        if (len > 0) {
            for (uint256 i = 0; i < len; i++) {
                _weights[i] = 1;
                _pairKeysAdd[i] = BGLS.decodeG1(ist.addedG1PubKey[i]);
            }
        }
        if (_pairKeysAdd.length > 0) {
            for (uint256 i = 0; i < _pairKeysAdd.length; i++) {
                v.pairKeys.push(BGLS.G1({x: _pairKeysAdd[i].x, y: _pairKeysAdd[i].y}));
                v.weights.push(_weights[i]);
                _weight = _weight + _weights[i];
            }
        }
        v.threshold = _weight - _weight / 3;

        emit MapUpdateValidators(_pairKeysAdd, _weights, epoch, bits);
    }


    /** internal view *********************************************************/

    function _verifyMptProof(
        bytes32 receiptHash,
        ReceiptProof memory _receiptProof
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
        logsHash = verifyTool.decodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
        (success, message) = verifyTool.getVerifyTrieProof(
            receiptHash,
            _receiptProof.keyIndex,
            _receiptProof.proof,
            _receiptProof.txReceiptRlp.receiptRlp,
            _receiptProof.txReceiptRlp.receiptType
        );
        if (!success) {
            message = "Mpt verification failed";
            return (success, message, "");
        }
    }

    function _verifyProofData(
        ReceiptProof memory _receiptProof
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
        (uint256 min, uint256 max) = verifiableHeaderRange();
        uint256 height = _receiptProof.header.number;
        if (height <= min || height >= max) {
            message = "Out of verify range";
            return (false, message, logsHash);
        }

        (success, message, logsHash) = _verifyMptProof(bytes32(_receiptProof.header.receiptHash), _receiptProof);
        if (!success) {
            return (success, message, "");
        }
        success = _verifyHeaderSig(_receiptProof.header, _receiptProof.ist, _receiptProof.aggPk);
        if (!success) {
            message = "VerifyHeaderSig failed";
            return (success, message, logsHash);
        }
        return (success, message, logsHash);
    }


    function _verifyHeaderSig(
        ILightNodePoint.blockHeader memory _bh,
        ILightNodePoint.istanbulExtra memory ist,
        BGLS.G2 memory _aggPk
    ) internal view returns (bool success) {
        bytes32 extraDataPre = bytes32(_bh.extraData);
        (bytes memory deleteAggBytes, bytes memory deleteSealAndAggBytes) = verifyTool.manageAgg(ist);
        deleteAggBytes = abi.encodePacked(extraDataPre, deleteAggBytes);
        deleteSealAndAggBytes = abi.encodePacked(extraDataPre, deleteSealAndAggBytes);

        (bytes memory deleteAggHeaderBytes, bytes memory deleteSealAndAggHeaderBytes) = verifyTool.encodeHeader(
            _bh,
            deleteAggBytes,
            deleteSealAndAggBytes
        );

        (success, ) = verifyTool.verifyHeader(_bh.coinbase, ist.seal, deleteSealAndAggHeaderBytes);
        if (!success) return success;

        return checkSig(_bh.number, ist, _aggPk, deleteAggHeaderBytes);
    }


    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    function checkSig(
        uint256 _blockNumber,
        ILightNodePoint.istanbulExtra memory _ist,
        BGLS.G2 memory _aggPk,
        bytes memory _headerWithoutAgg
    ) internal view returns (bool) {
        uint256 epoch = _getEpochByNumber(_blockNumber);
        bytes memory message = getPrepareCommittedSeal(_headerWithoutAgg, _ist.aggregatedSeal.round);
        bytes memory bits = abi.encodePacked(_ist.aggregatedSeal.bitmap);
        BGLS.G1 memory sig = BGLS.decodeG1(_ist.aggregatedSeal.signature);

        uint256 id = _getEpochId(epoch);
        Epoch memory v = epochs[id];
        return
            // isQuorum(bits, v.weights, v.threshold) &&
            BGLS.checkAggPk(bits, _aggPk, v.pairKeys, v.weights, v.threshold) &&
            BGLS.checkSignature(message, sig, _aggPk);
        // return checkSigTag(bits, message, sig, _aggPk, epoch);
    }

    function _getEpochId(uint256 epoch) internal view returns (uint256) {
        return epoch % maxEpochs;
    }

    function _getPreEpochId(uint256 epoch) internal view returns (uint256) {
        uint256 id = _getEpochId(epoch);
        if (id == 0) {
            return maxEpochs - 1;
        } else {
            return id - 1;
        }
    }

    function isQuorum(bytes memory bits, uint256[] memory weights, uint256 threshold) internal pure returns (bool) {
        uint256 weight = 0;
        for (uint256 i = 0; i < weights.length; i++) {
            if (BGLS.chkBit(bits, i)) weight += weights[i];
        }
        return weight >= threshold;
    }


    function getPrepareCommittedSeal(
        bytes memory _headerWithoutAgg,
        uint256 _round
    ) internal pure returns (bytes memory result) {
        bytes32 hash = keccak256(_headerWithoutAgg);
        if (_round == 0) {
            result = abi.encodePacked(hash, uint8(2));
        } else {
            result = abi.encodePacked(hash, getLengthInBytes(_round), uint8(2));
        }
    }

    function getLengthInBytes(uint256 num) internal pure returns (bytes memory) {
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

    function _getEpochByNumber(uint256 blockNumber) internal view returns (uint256) {
        if (blockNumber % epochSize == 0) {
            return blockNumber / epochSize;
        }
        return blockNumber / epochSize + 1;
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightNode only Admin can upgrade");
    }

    function changeAdmin() public {
        require(_pendingAdmin == msg.sender, "Only pendingAdmin");
        emit AdminTransferred(_getAdmin(), _pendingAdmin);
        _changeAdmin(_pendingAdmin);
        _pendingAdmin = address(0);
    }

    function pendingAdmin() external view returns (address) {
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
