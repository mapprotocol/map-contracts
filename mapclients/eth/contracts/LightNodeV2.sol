// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/IVerifyToolV2.sol";
import "./bls/BGLS.sol";

contract LightNodeV2 is UUPSUpgradeable, Initializable, ILightVerifier {

    uint256 internal constant MIN_HEADER_LENGTH = 577;
    uint256 internal constant MAX_VERIFIABLE_BLOCK_NUMBER = 1500000;

    address private _pendingAdmin;

    // startHeight (8bytes) | headerHeight (8bytes) | maxEpochs (4bytes) | epochSize (4bytes)
    uint256 private stateSlot;

    IVerifyToolV2 public verifyTool;

    mapping(uint256 => Epoch) private epochs;
    mapping(uint256 => bytes32) private cachedReceiptRoot;

    struct ReceiptProofV2 {
        uint256 blockNumber;

        bytes aggHeader;    // RLP header for aggregated sign, remove agg signs
        bytes signHeader;   // RLP header for proposer sign, remove seal and agg signs
        IVerifyToolV2.istanbulExtra ist;
        uint256[] pairKeys; // all validator G1 pk + G2 agg pk
        BGLS.G2 aggPk;

        uint256 receiptType;
        bytes receiptRlp;
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

    event UpdateBlockHeader(address indexed maintainer, uint256 indexed blockHeight);

    event InitializeValidators(uint256 _threshold, uint256[] _pairKeys, uint256[] _weights, uint256 epoch);
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
        uint256 maxEpochs = MAX_VERIFIABLE_BLOCK_NUMBER / _epochSize;
        uint256 lastHeight = (_epoch - 1) * _epochSize;
        _setNodeState(lastHeight, lastHeight, maxEpochs, _epochSize);

        uint256 index = _getEpochIndex(_epoch, maxEpochs);

        Epoch storage epoch = epochs[index];
        epoch.threshold = uint64(_threshold);
        epoch.epoch = uint128(_epoch);
        epoch.pairKeyHash = _getKeyHash(_pairKeys, _pairKeys.length);

        (epoch.aggKey[0], epoch.aggKey[1]) = BGLS.sumAllPoints(_pairKeys, _pairKeys.length / 2);

        verifyTool = IVerifyToolV2(_verifyTool);

        emit InitializeValidators(_threshold, _pairKeys, _weights, _epoch);
    }

    function setVerifyTool(address _verifyTool) external onlyOwner {
        verifyTool = IVerifyToolV2(_verifyTool);
        emit NewVerifyTool(_verifyTool);
    }

    function updateBlockHeader(
        uint256 blockNumber,
        bytes memory aggHeader,
        bytes calldata signHeader,
        IVerifyToolV2.istanbulExtra memory ist,
        BGLS.G2 memory aggPk,
        uint256[] memory pairKeys
    ) external {
        require(aggHeader.length > MIN_HEADER_LENGTH, "Invalid agg header");
        require(signHeader.length > MIN_HEADER_LENGTH, "Invalid sign header");

        uint256 index = _checkBlockNumber(blockNumber, true);
        Epoch memory epoch = epochs[index];

        require(_getKeyHash(pairKeys, pairKeys.length) == epoch.pairKeyHash, "keys hash error");

        (bool success, string memory message, address coinbase, ) = verifyTool.checkHeader(blockNumber, aggHeader, signHeader, ist, true, false);
        require(success, message);

        _verifyHeaderSig(epoch, aggHeader, signHeader, coinbase, pairKeys, ist, aggPk);

        _updateValidators(blockNumber, epoch, pairKeys, ist);

        emit UpdateBlockHeader(msg.sender, blockNumber);
    }

    function verifyProofDataWithCache(
        bool _cache,
        uint256 _logIndex,
        bytes memory _receiptProofBytes
    ) external override returns (bool success, string memory message, txLog memory log) {
        ReceiptProofV2 memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProofV2));

        bytes32 receiptRoot;
        if (_cache) {
            receiptRoot = cachedReceiptRoot[_receiptProof.blockNumber];
            if (receiptRoot != bytes32("")) {
                return _verifyMptProofWithLog(_logIndex, receiptRoot, _receiptProof);
            }
        }
        receiptRoot = _verifyHeaderProof(_receiptProof);
        if (_cache) {
            cachedReceiptRoot[_receiptProof.blockNumber] = receiptRoot;
        }

        return _verifyMptProofWithLog(_logIndex, receiptRoot, _receiptProof);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProofBytes
    ) external override returns (bool success, string memory message, bytes memory logsHash) {
        ReceiptProofV2 memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProofV2));

        bytes32 receiptRoot = cachedReceiptRoot[_receiptProof.blockNumber];
        if (receiptRoot != bytes32("")) {
            return _verifyMptProof(receiptRoot, _receiptProof);
        }
        receiptRoot = _verifyHeaderProof(_receiptProof);
        cachedReceiptRoot[_receiptProof.blockNumber] = receiptRoot;

        return _verifyMptProof(receiptRoot, _receiptProof);
    }

    function notifyLightClient(address _from, bytes memory _data) external override {
        emit ClientNotifySend(_from, block.number, _data);
    }

    /** view *********************************************************/

    function verifyProofData(
        bytes memory _receiptProofBytes
    ) external view override returns (bool success, string memory message, bytes memory logsHash) {
        ReceiptProofV2 memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProofV2));

        bytes32 receiptRoot = _verifyHeaderProof(_receiptProof);

        return _verifyMptProof(receiptRoot, _receiptProof);
    }

    function verifyProofData(
        uint256 _logIndex,
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, txLog memory log) {
        ReceiptProofV2 memory receiptProof = abi.decode(_receiptProof, (ReceiptProofV2));

        bytes32 receiptRoot = _verifyHeaderProof(receiptProof);
        return _verifyMptProofWithLog(_logIndex, receiptRoot, receiptProof);
    }

    function getData(bytes memory _receiptProofBytes) external pure returns (ReceiptProofV2 memory) {
        ReceiptProofV2 memory _receiptProof = abi.decode(_receiptProofBytes, (ReceiptProofV2));

        return _receiptProof;
    }

    function getBytes(ReceiptProofV2 memory _receiptProof) public pure returns (bytes memory) {
        return abi.encode(_receiptProof);
    }

    function headerHeight() external view returns (uint256 lastHeight) {
        (, lastHeight, , ) = _getNodeState();
    }

    function verifiableHeaderRange() public view override returns (uint256, uint256) {
        return _verifiableHeaderRange();
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        (uint256 start, uint256 end) = _verifiableHeaderRange();
        return start <= _blockHeight && _blockHeight <= end;
    }

    function _verifiableHeaderRange() internal view returns (uint256, uint256) {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();
        return _verifiableHeaderRange(startHeight, lastHeight, maxEpoch, epochSize);
    }

    function nodeType() external view override returns (uint256) {
        return 1;
    }

    function isCachedReceiptRoot(uint256 _blockHeight) external view returns (bool) {
        return (cachedReceiptRoot[_blockHeight] != bytes32(""));
    }

    /** internal *********************************************************/
    function _updateValidators(
        uint256 blockNumber,
        Epoch memory _preEpoch,
        uint256[] memory _pairKeys,
        IVerifyToolV2.istanbulExtra memory _ist
    ) internal {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();
        lastHeight = blockNumber;
        if (startHeight == 0) {
            startHeight = lastHeight - epochSize;
        }
        _setNodeState(startHeight, lastHeight, maxEpoch, epochSize);

        uint256 epoch = _preEpoch.epoch + 1;
        uint256 index = _getEpochIndex(epoch, maxEpoch);

        epochs[index].epoch = uint128(epoch);
        if (_ist.removeList == 0x00 && _ist.addedG1PubKey.length == 0) {
            epochs[index].threshold = _preEpoch.threshold;
            epochs[index].pairKeyHash = _preEpoch.pairKeyHash;

            epochs[index].aggKey[0] = _preEpoch.aggKey[0];
            epochs[index].aggKey[1] = _preEpoch.aggKey[1];

            emit UpdateValidators(epoch, _ist.removeList, _ist.addedG1PubKey);
            return;
        }

        uint256[] memory keys = new uint256[](_pairKeys.length + _ist.addedG1PubKey.length * 2);
        uint256 keyLength;
        uint256 weight = 0;

        uint256 keyLen = _pairKeys.length / 2;
        // if (_ist.removeList > 0x00)
        {
            for (uint256 i = 0; i < keyLen; i++) {
                if (!BGLS.chkBitmap(_ist.removeList, i)) {
                    keys[keyLength] = _pairKeys[2 * i];
                    keys[keyLength + 1] = _pairKeys[2 * i + 1];

                    keyLength += 2;
                    weight = weight + 1;
                }
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

        (epochs[index].aggKey[0], epochs[index].aggKey[1]) = BGLS.sumAllPoints(keys, keyLength / 2);

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
        ReceiptProofV2 memory _receiptProof
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
        (success, logsHash) = verifyTool.verifyTrieProof(
            receiptHash,
            _receiptProof.keyIndex,
            _receiptProof.proof,
            _receiptProof.receiptRlp,
            _receiptProof.receiptType
        );
        if (!success) {
            return (success, "MPT verification failed", logsHash);
        }
        return (success, "", logsHash);
    }

    function _verifyMptProofWithLog(
        uint256 _logIndex,
        bytes32 _receiptHash,
        ReceiptProofV2 memory _receiptProof
    ) internal view returns (bool success, string memory message, txLog memory log) {
        (success, log) = verifyTool.verifyTrieProofWithLog(_logIndex,
            _receiptHash,
            _receiptProof.keyIndex,
            _receiptProof.proof,
            _receiptProof.receiptRlp,
            _receiptProof.receiptType);

        if (!success) {
            return (success, "MPT verification failed", log);
        }

        return (success, "", log);
    }

    function _verifyHeaderProof(
        ReceiptProofV2 memory _receiptProof
    ) internal view returns (bytes32) {
        uint256 index = _checkBlockNumber(_receiptProof.blockNumber, false);
        Epoch memory epoch = epochs[index];
        require((_getKeyHash(_receiptProof.pairKeys, _receiptProof.pairKeys.length) == epoch.pairKeyHash),
            "keys hash error");

        (bool success, string memory message, address coinbase, bytes32 receiptRoot) = verifyTool.checkHeader(
            _receiptProof.blockNumber,
            _receiptProof.aggHeader,
            _receiptProof.signHeader,
            _receiptProof.ist,
            false,
            true
        );
        require(success, message);

        _verifyHeaderSig(epoch,
            _receiptProof.aggHeader,
            _receiptProof.signHeader,
            coinbase,
            _receiptProof.pairKeys,
            _receiptProof.ist,
            _receiptProof.aggPk);

        return receiptRoot;
    }

    function _verifyHeaderSig(
        Epoch memory _epoch,
        bytes memory _header,
        bytes memory _signHeader,
        address _coinbase,
        uint256[] memory _pairKeys,
        IVerifyToolV2.istanbulExtra memory ist,
        BGLS.G2 memory _aggPk
    ) internal view returns (bool success) {
        bytes32 headerHash = keccak256(_signHeader);
        success = verifyTool.verifyHeaderHash(_coinbase, ist.seal, headerHash);
        require(success, "Invalid header hash");

        headerHash = keccak256(_header);
        success = checkSig(headerHash, _epoch, _aggPk, _pairKeys, ist);
        require(success, "check sig failed");

        return success;
    }

    // aggPk2, sig1 --> in contract: check aggPk2 is valid with bits by summing points in G2
    // how to check aggPk2 is valid --> via checkAggPk
    function checkSig(
        bytes32 _headerHash,
        Epoch memory _epoch,
        BGLS.G2 memory _aggPk,
        uint256[] memory _pairKeys,
        IVerifyToolV2.istanbulExtra memory _ist
    ) internal view returns (bool) {
        bytes memory message = getPrepareCommittedSeal(_headerHash, _ist.aggregatedSeal.round);

        return
            BGLS.checkAggPk2(_epoch.aggKey, _ist.aggregatedSeal.bitmap, _aggPk, _pairKeys, _epoch.threshold) &&
            BGLS.checkSignature(message, _ist.aggregatedSeal.signature, _aggPk);
    }

    function getPrepareCommittedSeal(
        bytes32 hash,
        uint256 _round
    ) internal pure returns (bytes memory result) {
        // bytes32 hash = keccak256(_headerWithoutAgg);
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

    function _getEpochIndex(uint256 epoch, uint256 maxEpochs) internal pure returns (uint256) {
        return epoch % maxEpochs;
    }

    function _checkBlockNumber(uint256 _blockNumber, bool _updateHeader) internal view returns (uint256) {
        (uint256 startHeight, uint256 lastHeight, uint256 maxEpoch, uint256 epochSize) = _getNodeState();

        (uint256 min, uint256 max) = _verifiableHeaderRange(startHeight, lastHeight, maxEpoch, epochSize);
        require(_blockNumber >= min && _blockNumber <= max, "Out of verify range");

        if (_updateHeader) {
            require(_blockNumber % epochSize == 0, "Header number is error");
            require(_blockNumber - epochSize == lastHeight, "Header is have");
        }

        uint256 epochId = _getEpochByNumber(_blockNumber, epochSize);

        return _getEpochIndex(epochId, maxEpoch);
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
