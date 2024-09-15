// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/ILightNode.sol";
import "./bls/BGLS.sol";
import "./interface/IVerifyTool.sol";
import "hardhat/console.sol";

contract LightNodeV2 is UUPSUpgradeable, Initializable, ILightNode {
    address private _pendingAdmin;

    // uint256 public maxEpochs; // max epoch number
    // uint256 public epochSize; // every epoch block number
    // uint256 public startHeight; // init epoch start block number
    // uint256 public headerHeight; // last update block number

    // startHeight (8bytes) | headerHeight (8bytes) | maxEpochs (4bytes) | epochSize (4bytes)
    uint256 private stateSlot;

    IVerifyTool public verifyTool;
    // address[] public validatorAddress;
    //Epoch[] public epochs;
    mapping(uint256 => Epoch) public epochs;
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
        uint256[] pairKeys;
        bytes keyIndex;
        bytes[] proof;
    }

    struct Epoch {
        uint128 epoch;
        uint64  validatorNumber;
        uint64  threshold; // bft, > 2/3,  if  \sum weights = 100, threshold = 67
        bytes32 pairKeyHash; // <-- validators, pubkey G1,   (s, s * g2)   s * g1
        uint256[2] aggKey; // agg G1 key, not used now
    }

    event MapInitializeValidators(uint256 _threshold, uint256[] _pairKeys, uint256[] _weights, uint256 epoch);
    event UpdateValidators(uint256 epoch, uint256 removeBits, bytes[] _pairKeysAdd);
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
        uint256[] memory _pairKeys,
        uint256[] memory _weights,
        uint256 _epoch,
        uint256 _epochSize,
        address _verifyTool,
        address _owner
    ) external initializer {
        require(_epoch > 1, "Error initializing epoch");
        require(_threshold < 1000, "invalid threshold");

        _changeAdmin(_owner);
        uint256 maxEpochs = 1500000 / _epochSize;
        uint256 lastHeight = (_epoch - 1) * _epochSize;
        //startHeight = headerHeight;
        //epochSize = _epochSize;
        _setNodeState(lastHeight, lastHeight, maxEpochs, _epochSize);
        _setStateInternal(_threshold, _pairKeys, _epoch, maxEpochs);
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
        BGLS.G2 memory aggPk,
        uint256[] memory pairKeys
    ) external {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();

        require(bh.number % epochSize == 0, "Header number is error");
        require(bh.number - epochSize == lastHeight, "Header is have");
        lastHeight = bh.number;
        if (startHeight == 0) {
            startHeight = lastHeight - epochSize;
        }

        _setNodeState(startHeight, lastHeight, maxEpoch, epochSize);

        uint256 epochId = _getEpochByNumber(bh.number, epochSize);
        uint256 index = _getEpochIndex(epochId, maxEpoch);
        Epoch memory epoch = epochs[index];
        require(_getKeyHash(pairKeys, pairKeys.length) == epoch.pairKeyHash, "pair key hash error");

        bool success = _verifyHeaderSig(epoch, pairKeys, bh, ist, aggPk);
        require(success, "CheckSig error");

        _updateValidators(maxEpoch, epoch, pairKeys, ist);

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

    function getData(bytes memory _receiptProofBytes) external pure returns (ReceiptProof memory) {
        ReceiptProof memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProof));

        return _receiptProof;
    }

    function getBytes(ReceiptProof memory _receiptProof) public pure returns (bytes memory) {
        return abi.encode(_receiptProof);
    }

    function headerHeight() external view returns (uint256 lastHeight) {
        (, lastHeight, , ) = _getNodeState();
    }

    function verifiableHeaderRange() public view override returns (uint256, uint256) {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();
        return _verifiableHeaderRange(startHeight, lastHeight, maxEpoch, epochSize);
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();

        (uint256 start, uint256 end) = _verifiableHeaderRange(startHeight, lastHeight, maxEpoch, epochSize);
        return start <= _blockHeight && _blockHeight <= end;
    }

    function nodeType() external view override returns (uint256) {
        return 1;
    }

    /** internal *********************************************************/

    function _setStateInternal(uint256 _threshold, uint256[] memory _pairKeys, uint256 _epoch, uint256 _maxEpochs) internal {
        uint256 index = _getEpochIndex(_epoch, _maxEpochs);
        epochs[index].threshold = uint64(_threshold);
        epochs[index].epoch = uint128(_epoch);
        // epochs[id].pairKeyHash = keccak256(abi.encode(_pairKeys));
        epochs[index].pairKeyHash = _getKeyHash(_pairKeys, _pairKeys.length);

        // todo: update agg key
        //(epochs[index].aggKey[0], epochs[index].aggKey[1]) = BGLS.sumAllPoints(_pairKeys);
        //console.logUint(epochs[index].aggKey[0]);
        //console.logUint(epochs[index].aggKey[1]);
    }

    function _updateValidators(
        uint256 _maxEpochs,
        Epoch memory _preEpoch,
        uint256[] memory _pairKeys,
        IVerifyTool.istanbulExtra memory _ist
    ) internal {
        uint256 epoch = _preEpoch.epoch + 1;
        uint256 index = _getEpochIndex(epoch, _maxEpochs);

        epochs[index].epoch = uint128(epoch);
        if (_ist.removeList == 0x00 && _ist.addedG1PubKey.length == 0) {
            epochs[index].threshold = _preEpoch.threshold;
            epochs[index].pairKeyHash = _preEpoch.pairKeyHash;

            //epochs[index].aggKey[0] = _preEpoch.aggKey[0];
            //epochs[index].aggKey[1] = _preEpoch.aggKey[1];

            emit UpdateValidators(epoch, _ist.removeList, _ist.addedG1PubKey);
            return;
        }

        uint256[] memory keys = new uint256[](_pairKeys.length + _ist.addedG1PubKey.length * 2);
        //uint256[] memory keys;
        uint256 keyLength;

        bytes memory bits = abi.encodePacked(_ist.removeList);
        uint256 weight = 0;
        uint256 keyLen = _pairKeys.length / 2;
        for (uint256 i = 0; i < keyLen; i++) {
            if (!BGLS.chkBit(bits, i)) {
                keys[keyLength] = _pairKeys[2 * i];
                keys[keyLength + 1] = _pairKeys[2 * i + 1];

                keyLength += 2;
                weight = weight + 1;
            }
        }

        keyLen = _ist.addedG1PubKey.length;
        for (uint256 i = 0; i < keyLen; i++) {
            bytes memory g1 = _ist.addedG1PubKey[i];

            bytes32 x;
            bytes32 y;
            assembly {
                x := mload(add(g1, 32))
                y := mload(add(g1, 64))
            }

            keys[keyLength] = uint256(x);
            keys[keyLength + 1] = uint256(y);
            keyLength += 2;
            weight = weight + 1;
        }

        epochs[index].pairKeyHash = _getKeyHash(keys, keyLength);
        epochs[index].threshold = uint64(weight - weight / 3);

        //(epochs[index].aggKey[0], epochs[index].aggKey[1]) = BGLS.sumAllPoints(_pairKeys);

        emit UpdateValidators(epoch, _ist.removeList, _ist.addedG1PubKey);
    }

    /** internal view *********************************************************/

    function _getKeyHash(
        uint256[] memory _pairKeys,
        uint256 keyLen
    ) internal pure returns (bytes32 result) {
        uint256 len = 0x20 * keyLen;
        uint256 ptr;
        assembly {
            // skip the array length
            ptr := add(0x20, _pairKeys)
            result := keccak256(ptr, len)
        }
    }

    function _getNodeState() internal view returns (uint256 start, uint256 end, uint256 maxEpoch, uint256 epochSize) {
        uint256 nodeSlot = stateSlot;
        start = nodeSlot >> 192;
        end = (nodeSlot >> 128) & 0xFFFFFFFF;
        maxEpoch = (nodeSlot >> 32) & 0xFFFFFFFF;
        epochSize = nodeSlot & 0xFFFFFFFF;
    }

    function _setNodeState(uint256 start, uint256 end, uint256 maxEpoch, uint256 epochSize) internal {
        stateSlot = (start << 192) | (end << 128) | (maxEpoch << 32) | epochSize;
    }

    function _verifiableHeaderRange(
        uint256 _startHeight,
        uint256 _headerHeight,
        uint256 _maxEpoch,
        uint256 _epochSize
    ) internal pure returns (uint256, uint256) {
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
        logsHash = verifyTool.unsafeDecodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
    }

    function _verifyProofData(
        ReceiptProof memory _receiptProof
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();
        {
            (uint256 min, uint256 max) = _verifiableHeaderRange(startHeight, lastHeight, maxEpoch, epochSize);
            // uint256 height = _receiptProof.header.number;
            if (_receiptProof.header.number <= min || _receiptProof.header.number >= max) {
                message = "Out of verify range";
                return (false, message, logsHash);
            }
        }

        Epoch memory epoch;
        {
            uint256 epochId = _getEpochByNumber(_receiptProof.header.number, epochSize);
            uint256 index = _getEpochIndex(epochId, maxEpoch);
            epoch = epochs[index];
        }
        if (_getKeyHash(_receiptProof.pairKeys, _receiptProof.pairKeys.length) != epoch.pairKeyHash) {
            return (false, message, bytes("pair key hash error"));
        }

        success = _verifyHeaderSig(
            epoch,
            _receiptProof.pairKeys,
            _receiptProof.header,
            _receiptProof.ist,
            _receiptProof.aggPk
        );
        if (!success) {
            message = "VerifyHeaderSig failed";
            return (success, message, logsHash);
        }

        (success, message, logsHash) = _verifyMptProof(bytes32(_receiptProof.header.receiptHash), _receiptProof);
        if (!success) {
            return (success, message, "");
        }

        return (success, message, logsHash);
    }

    function _verifyHeaderSig(
        Epoch memory _epoch,
        uint256[] memory _pairKeys,
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

        return checkSig(_epoch, _pairKeys, ist, _aggPk, deleteAggHeaderBytes);
    }

    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    function checkSig(
        Epoch memory _epoch,
        uint256[] memory _pairKeys,
        IVerifyTool.istanbulExtra memory _ist,
        BGLS.G2 memory _aggPk,
        bytes memory _headerWithoutAgg
    ) internal view returns (bool) {
        bytes memory message = getPrepareCommittedSeal(_headerWithoutAgg, _ist.aggregatedSeal.round);
        bytes memory bits = abi.encodePacked(_ist.aggregatedSeal.bitmap);

        return
            BGLS.checkAggPk(bits, _aggPk, _pairKeys, _epoch.threshold) &&
            BGLS.checkSignature(message, _ist.aggregatedSeal.signature, _aggPk);
    }

    function _getEpochIndex(uint256 epoch, uint256 maxEpochs) internal pure returns (uint256) {
        return epoch % maxEpochs;
    }

    /*
    function _getPreEpochId(uint256 epoch) internal view returns (uint256) {
        uint256 id = _getEpochId(epoch);
        if (id == 0) {
            return maxEpochs - 1;
        } else {
            return id - 1;
        }
    }*/

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

    function _getEpochByNumber(uint256 blockNumber, uint256 epochSize) internal pure returns (uint256) {
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
