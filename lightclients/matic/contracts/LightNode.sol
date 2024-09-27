// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "./lib/Verify.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    uint256 public constant EPOCH_NUM = 64;

    uint256 internal constant MAX_SAVED_EPOCH_NUM = 121500;

    uint256 internal constant ADDRESS_LENGTH = 20;

    address public mptVerify;

    uint256 public minValidBlocknum;

    uint256 public minEpochBlockExtraDataLen;

    mapping(uint256 => bytes) public validators;

    uint256 internal _lastSyncedBlock;

    uint256 public chainId;

    uint256 public confirms;

    address private _pendingAdmin;

    event SetMptVerify(address newMptVerify);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event AdminTransferred(address indexed previous, address indexed newAdmin);

    struct ProofData {
        Verify.BlockHeader[] headers;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    constructor() {}

    function initialize(
        uint256 _chainId,
        uint256 _minEpochBlockExtraDataLen,
        address _controller,
        address _mptVerify,
        uint256 _confirms,
        Verify.BlockHeader calldata _header
    ) external initializer {
        require(_chainId > 0, "invalid _chainId");
        require(_confirms > 0, "invalid _confirms");
        require(_minEpochBlockExtraDataLen > 0, "_minEpochBlockExtraDataLen is zero");
        require(minEpochBlockExtraDataLen == 0, "already initialized");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        chainId = _chainId;
        mptVerify = _mptVerify;
        confirms = _confirms;
        _changeAdmin(_controller);
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
        _initBlock(_header);
    }

    function setMptVerify(address _verifier) external onlyOwner {
        require(_verifier != address(0), "LightNode: verifier is the zero address");
        mptVerify = _verifier;
        emit SetMptVerify(_verifier);
    }

    function togglePause(bool _flag) external onlyOwner returns (bool) {
        if (_flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function updateBlockHeader(bytes memory _blockHeadersBytes) external override whenNotPaused {
        Verify.BlockHeader[] memory _blockHeaders = abi.decode(_blockHeadersBytes, (Verify.BlockHeader[]));

        require(confirms > 0, "light node uninitialized");

        require(_blockHeaders.length == confirms, "proof headers not enough");

        _lastSyncedBlock += Verify._getEpochNumber(chainId, _lastSyncedBlock + 1);

        require(_blockHeaders[0].number == _lastSyncedBlock, "invalid start block");

        uint256 epoch = Verify._getEpochNumber(chainId, _lastSyncedBlock + 1);

        // index 0 header verify by pre validators others by index 0 getValidators
        validators[(_lastSyncedBlock + 1) / epoch] = Verify._getValidators(
            _blockHeaders[0].extraData,
            _blockHeaders[0].number
        );
        (bool result, string memory message) = _verifyBlockHeaders(_blockHeaders);
        require(result, message);

        _removeExcessEpochValidators();

        emit UpdateBlockHeader(tx.origin, _blockHeaders[0].number);
    }

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        return _verifyProofData(proof.receiptProof, proof.headers);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        return _verifyProofData(proof.receiptProof, proof.headers);
    }

    function verifyProofData(
        uint256 _logIndex,
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, txLog memory log){
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        return _validateProofWithLog(_logIndex, proof.receiptProof, proof.headers);
    }

    function verifyProofDataWithCache(
        bool _cache,
        uint256 _logIndex,
        bytes memory _receiptProofBytes
    ) external override returns (bool success, string memory message, txLog memory log){
        ProofData memory proof = abi.decode(_receiptProofBytes, (ProofData));
        return _validateProofWithLog(_logIndex, proof.receiptProof, proof.headers);
    }

    function _verifyProofData(
        Verify.ReceiptProof memory receiptProof,
        Verify.BlockHeader[] memory headers
    ) private view returns (bool success, string memory message, bytes memory logs) {
        bytes32 rootHash;
        (rootHash, success, message) = _getReceiptsRoot(headers);
        if (success) {
            (success, logs) = Verify._validateProof(rootHash, receiptProof, mptVerify);
            if (!success) {
                message = "mpt verification failed";
            }
        }
    }

    function _validateProofWithLog(
        uint256 _logIndex,
        Verify.ReceiptProof memory receiptProof,
        Verify.BlockHeader[] memory headers
    ) private view returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        bytes32 rootHash;
        (rootHash, success, message) = _getReceiptsRoot(headers);
        if (success) {
            (success, log) = Verify._validateProofWithLog(_logIndex, rootHash, receiptProof, mptVerify);
            if (!success) {
                message = "mpt verification failed";
            }
        }
    }

    function _getReceiptsRoot(Verify.BlockHeader[] memory headers) private view returns(bytes32 rootHash, bool success, string memory message){
        require(confirms > 0, "light node uninitialized");
        require(headers.length == confirms, "proof headers not enough");
        require(
            headers[0].number >= minValidBlocknum && headers[headers.length - 1].number <= maxCanVerifyNum(),
            "Out of verify range"
        );
        (success, message) = _verifyBlockHeaders(headers);

        if(success) rootHash = bytes32(headers[0].receiptsRoot);
    }

    function _initBlock(Verify.BlockHeader memory _header) internal {
        require(_lastSyncedBlock == 0, "already init");
        uint256 epoch = Verify._getEpochNumber(chainId, _header.number + 1);
        require((_header.number + 1) % epoch == 0, "invalid init block");
        bytes memory validator = Verify._getValidators(_header.extraData, _header.number);
        require(validator.length >= ADDRESS_LENGTH, "no validator init");

        validators[(_header.number + 1) / epoch] = validator;

        _lastSyncedBlock = _header.number;

        minValidBlocknum = _header.number + 1;
    }

    function _verifyBlockHeaders(
        Verify.BlockHeader[] memory _blockHeaders
    ) internal view returns (bool, string memory) {
        for (uint256 i = 0; i < _blockHeaders.length; i++) {
            if (i == 0) {
                if (!Verify._validateHeader(_blockHeaders[i], minEpochBlockExtraDataLen, _blockHeaders[i], chainId)) {
                    return (false, "invalid block");
                }
            } else {
                if (
                    !Verify._validateHeader(_blockHeaders[i], minEpochBlockExtraDataLen, _blockHeaders[i - 1], chainId)
                ) {
                    return (false, "invalid block");
                }
            }

            address signer = Verify._recoverSigner(_blockHeaders[i]);
            uint256 epoch = Verify._getEpochNumber(chainId, _blockHeaders[i].number);
            if (!Verify._containsValidator(validators[_blockHeaders[i].number / epoch], signer)) {
                return (false, "invalid signer");
            }
        }

        return (true, "");
    }

    function _removeExcessEpochValidators() internal {
        if (_lastSyncedBlock < EPOCH_NUM * MAX_SAVED_EPOCH_NUM) {
            return;
        }
        uint256 remove = _lastSyncedBlock - EPOCH_NUM * MAX_SAVED_EPOCH_NUM;

        uint256 epoch = Verify._getEpochNumber(chainId, remove);

        if (remove + epoch > minValidBlocknum && validators[(remove + 1) / epoch].length > 0) {
            minValidBlocknum = remove + epoch + 1;
            delete validators[(remove + 1) / epoch];
        }
    }

    function getBytes(ProofData calldata _proof) external pure returns (bytes memory) {
        return abi.encode(_proof);
    }

    function getHeadersBytes(Verify.BlockHeader[] calldata _blockHeaders) external pure returns (bytes memory) {
        return abi.encode(_blockHeaders);
    }

    function headerHeight() external view override returns (uint256) {
        return _lastSyncedBlock;
    }

    function maxCanVerifyNum() public view returns (uint256) {
        return _lastSyncedBlock + Verify._getEpochNumber(chainId, _lastSyncedBlock + 1);
    }

    function verifiableHeaderRange() external view override returns (uint256, uint256) {
        return (minValidBlocknum, maxCanVerifyNum());
    }

    function notifyLightClient(address _from, bytes memory _data) external override {
        emit ClientNotifySend(_from, block.number, _data);
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        return minValidBlocknum <= _blockHeight && _blockHeight <= maxCanVerifyNum();
    }

    function nodeType() external view override returns (uint256) {
        // return this chain light node type on target chain
        // 1 default light client
        // 2 zk light client
        // 3 oracle client
        return 1;
    }

    function updateLightClient(bytes memory) external pure override {}

    function clientState() external pure override returns (bytes memory) {}

    function finalizedState(bytes memory) external pure override returns (bytes memory) {}

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }

    function changeAdmin() external {
        require(_pendingAdmin == msg.sender, "only pendingAdmin");
        emit AdminTransferred(_getAdmin(), _pendingAdmin);
        _changeAdmin(_pendingAdmin);
    }

    function pendingAdmin() external view returns (address) {
        return _pendingAdmin;
    }

    function setPendingAdmin(address pendingAdmin_) external onlyOwner {
        require(pendingAdmin_ != address(0), "Ownable: pendingAdmin is the zero address");
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
