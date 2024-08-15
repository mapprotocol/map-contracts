// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/ILightNode.sol";
import "./bls/BGLS.sol";
import "./interface/IVerifyTool.sol";

contract LightNode is UUPSUpgradeable, Initializable, ILightNode {
    address private _pendingAdmin;

    uint256 public maxEpochs; // max epoch number
    uint256 public epochSize; // every epoch block number

    uint256 public startHeight;     // init epoch start block number
    uint256 public headerHeight;    // last update block number
    // address[] public validatorAddress;
    Epoch[] public epochs;
    IVerifyTool public verifyTool;

    mapping(uint256 => bytes32) private cachedReceiptRoot;

    struct TxReceiptRlp {
        uint256 receiptType;
        bytes receiptRlp;
    }

    struct ReceiptProof {
        IVerifyTool.blockHeader header;
        IVerifyTool.istanbulExtra ist;
        BGLS.G2 aggPk;
        TxReceiptRlp txReceiptRlp;
        bytes keyIndex;
        bytes[] proof;
    }

    struct Epoch {
        uint256 epoch;
        uint256 threshold; // bft, > 2/3,  if  \sum weights = 100, threshold = 67
        uint256[2] aggKey;  // agg G1 key, not used now
        uint256[] pairKeys; // <-- validators, pubkey G1,   (s, s * g2)   s * g1
        uint256[] weights; // voting power, not used now
    }

    event MapInitializeValidators(uint256 _threshold, BGLS.G1[] _pairKeys, uint256[] _weights, uint256 epoch);
    event MapUpdateValidators(bytes[] _pairKeysAdd, uint256 epoch, bytes bits);
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
        address _verifyTool,
        address _owner
    ) external initializer {
        require(_epoch > 1, "Error initializing epoch");
        _changeAdmin(_owner);
        maxEpochs = 1500000 / _epochSize;
        headerHeight = (_epoch - 1) * _epochSize;
        startHeight = headerHeight;
        epochSize = _epochSize;
        // validatorAddress = _validatorAddress;
        // init all epochs
        for (uint256 i = 0; i < maxEpochs; i++) {
            epochs.push(
                Epoch({
                    pairKeys: new uint256[](0),
                    weights: new uint256[](0),
                    aggKey: [uint256(0), uint256(0)],
                    threshold: 0,
                    epoch: 0
                })
            );
        }
        setStateInternal(_threshold, _pairKeys, _weights, _epoch);
        verifyTool = IVerifyTool(_verifyTool);

        emit MapInitializeValidators(_threshold, _pairKeys, _weights, _epoch);
    }

    function setVerifyTool(address _verifyTool) external onlyOwner {
        verifyTool = IVerifyTool(_verifyTool);
        emit NewVerifyTool(_verifyTool);
    }

    function updateBlockHeader(
        IVerifyTool.blockHeader memory bh,
        IVerifyTool.istanbulExtra memory ist,
        BGLS.G2 memory aggPk
    ) external {
        require(bh.number % epochSize == 0, "Header number is error");
        require(bh.number - epochSize == headerHeight, "Header is have");
        headerHeight = bh.number;
        if (startHeight == 0) {
            startHeight = headerHeight - epochSize;
        }

        uint256 epoch = _getEpochByNumber(bh.number);
        uint256 id = _getEpochId(epoch);
        //Epoch memory v = epochs[id];
        Epoch memory v;
        v.epoch = epochs[id].epoch;
        // v.threshold = epochs[id].threshold;
        v.pairKeys = epochs[id].pairKeys;

        uint256 weight = v.pairKeys.length / 2;
        v.threshold = weight - weight / 3;

        bool success = _verifyHeaderSig(v, bh, ist, aggPk);
        require(success, "CheckSig error");

        _updateValidators(v, ist);

        emit UpdateBlockHeader(msg.sender, bh.number);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProofBytes
    ) external override returns (bool success, string memory message, bytes memory logsHash) {
        ReceiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProof));

        bytes32 receiptRoot = cachedReceiptRoot[_receiptProof.header.number];
        if (receiptRoot != bytes32("")) {
            return _verifyMptProof(receiptRoot, _receiptProof);
        } else {
            (success, message, logsHash) = _verifyProofData(_receiptProof);
            if (success) {
                cachedReceiptRoot[_receiptProof.header.number] = bytes32(_receiptProof.header.receiptHash);
            }
        }
    }

    function notifyLightClient(address _from, bytes memory _data) external override {
        emit ClientNotifySend(_from, block.number, _data);
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

    function getValidators(uint256 id) public view returns (uint256[] memory) {
        return epochs[id].pairKeys;
    }

    function getBytes(ReceiptProof memory _receiptProof) public pure returns (bytes memory) {
        return abi.encode(_receiptProof);
    }

    function verifiableHeaderRange() public view override returns (uint256, uint256) {
        return _verifiableHeaderRange(startHeight, headerHeight, maxEpochs, epochSize);
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        (uint256 start, uint256 end) = _verifiableHeaderRange(startHeight, headerHeight, maxEpochs, epochSize);
        return start <= _blockHeight && _blockHeight <= end;
    }

    function nodeType() external view override returns (uint256) {
        return 1;
    }

    /** internal *********************************************************/



    function setStateInternal(
        uint256 _threshold,
        BGLS.G1[] memory _pairKeys,
        uint256[] memory _weights,
        uint256 _epoch
    ) internal {
        require(_pairKeys.length == _weights.length, "Mismatch arg");
        uint256 id = _getEpochId(_epoch);
        Epoch storage v = epochs[id];

        for (uint256 i = 0; i < _pairKeys.length; i++) {
            v.pairKeys.push(_pairKeys[i].x);
            v.pairKeys.push(_pairKeys[i].y);
            // v.weights.push(_weights[i]);
        }

        v.threshold = _threshold;
        v.epoch = _epoch;
    }

    function _updateValidators(Epoch memory _preEpoch, IVerifyTool.istanbulExtra memory _ist) internal {
        bytes memory bits = abi.encodePacked(_ist.removeList);

        uint256 epoch = _preEpoch.epoch + 1;
        //uint256 idPre = _getPreEpochId(epoch);
        //Epoch memory vPre = epochs[idPre];
        uint256 id = _getEpochId(epoch);
        Epoch storage v = epochs[id];
        v.epoch = epoch;

        if (v.pairKeys.length > 0) {
            delete (v.pairKeys);
        }

        uint256 weight = 0;
        uint256 keyLen = _preEpoch.pairKeys.length / 2;
        for (uint256 i = 0; i < keyLen; i++) {
            if (!BGLS.chkBit(bits, i)) {
                v.pairKeys.push(_preEpoch.pairKeys[2 * i]);
                v.pairKeys.push(_preEpoch.pairKeys[2 * i + 1]);
                //v.weights.push(_preEpoch.weights[i]);
                weight = weight + 1;
            }
        }

        keyLen = _ist.addedG1PubKey.length;
        if (keyLen > 0) {
            bytes32 x;
            bytes32 y;
            for (uint256 i = 0; i < keyLen; i++) {
                bytes memory g1 = _ist.addedG1PubKey[i];
                assembly {
                    x := mload(add(g1, 32))
                    y := mload(add(g1, 64))
                }

                v.pairKeys.push(uint256(x));
                v.pairKeys.push(uint256(y));
                //v.weights.push(1);
                weight = weight + 1;
            }
        }
        v.threshold = weight - weight / 3;

        emit MapUpdateValidators(_ist.addedG1PubKey, epoch, bits);
    }

    /** internal view *********************************************************/

    function _verifiableHeaderRange(uint256 _startHeight, uint256 _headerHeight, uint256 _maxEpoch, uint256 _epochSize) internal pure returns (uint256, uint256) {
        uint256 start;
        if (_headerHeight > _maxEpoch * _epochSize) {
            start = _headerHeight - (_maxEpoch * _epochSize);
        }

        if (_startHeight > 0 && _startHeight > start) {
            start = _startHeight;
        }
        return (start, _headerHeight + _epochSize);
    }

    function _verifyMptProof(
        bytes32 receiptHash,
        ReceiptProof memory _receiptProof
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
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
        logsHash = verifyTool.decodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
    }

    function _verifyProofData(
        ReceiptProof memory _receiptProof
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
        (uint256 min, uint256 max) = _verifiableHeaderRange(startHeight, headerHeight, maxEpochs, epochSize);
        uint256 height = _receiptProof.header.number;
        if (height <= min || height >= max) {
            message = "Out of verify range";
            return (false, message, logsHash);
        }

        (success, message, logsHash) = _verifyMptProof(bytes32(_receiptProof.header.receiptHash), _receiptProof);
        if (!success) {
            return (success, message, "");
        }

        uint256 epoch = _getEpochByNumber(height);
        uint256 id = _getEpochId(epoch);
        //Epoch memory v = epochs[id];
        Epoch memory v;
        // v.threshold = epochs[id].threshold;
        v.pairKeys = epochs[id].pairKeys;
        uint256 weight = v.pairKeys.length / 2;
        v.threshold = weight - weight / 3;

        success = _verifyHeaderSig(v, _receiptProof.header, _receiptProof.ist, _receiptProof.aggPk);
        if (!success) {
            message = "VerifyHeaderSig failed";
            return (success, message, logsHash);
        }
        return (success, message, logsHash);
    }

    function _verifyHeaderSig(
        Epoch memory _epoch,
        IVerifyTool.blockHeader memory _bh,
        IVerifyTool.istanbulExtra memory ist,
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

        return checkSig(_epoch, ist, _aggPk, deleteAggHeaderBytes);
    }

    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    function checkSig(
        Epoch memory _epoch,
        IVerifyTool.istanbulExtra memory _ist,
        BGLS.G2 memory _aggPk,
        bytes memory _headerWithoutAgg
    ) internal view returns (bool) {
        bytes memory message = getPrepareCommittedSeal(_headerWithoutAgg, _ist.aggregatedSeal.round);
        bytes memory bits = abi.encodePacked(_ist.aggregatedSeal.bitmap);

        return
            BGLS.checkAggPk(bits, _aggPk, _epoch.pairKeys, _epoch.threshold) &&
            BGLS.checkSignature(message, _ist.aggregatedSeal.signature, _aggPk);
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
        uint256 epochLen = epochSize;
        if (blockNumber % epochLen == 0) {
            return blockNumber / epochLen;
        }
        return blockNumber / epochLen + 1;
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
        require(pendingAdmin_ != address(0), "pendingAdmin is the zero address");
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
