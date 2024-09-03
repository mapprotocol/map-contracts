// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import { Helper } from "./lib/Helper.sol";

import {
    BlockHeader,
    ReceiptProof,
    TxReceipt,
    TxLog,
    VoteData,
    VoteAttestation,
    UpdateHeader,
    ProofData
} from "./lib/Types.sol";

contract LightNodeV2 is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    using Helper for BlockHeader;

    uint256 internal constant EPOCH_NUM = 200;
    uint256 internal constant MAX_SAVED_EPOCH_NUM = 12960;
    uint256 internal constant ADDRESS_LENGTH = 20;

    address public mptVerify;
    uint256 public chainId;
    uint256 public minValidBlocknum;
    uint256 internal _lastSyncedBlock;
    address private _pendingAdmin;

    mapping(uint256 => bytes[]) public BLSPublicKeys;
    mapping(uint256 => bytes32) private cachedReceiptRoot;

    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event SetMptVerify(address newMptVerify);

    event AdminTransferred(address indexed previous, address indexed newAdmin);

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }
    

    constructor() {}

    function initialize(
        uint256 _chainId,
        address _controller,
        address _mptVerify,
        BlockHeader[2] calldata headers
    ) external initializer {
        require(chainId == 0, "already initialized");
        require(_chainId > 0, "_chainId is zero");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
        chainId = _chainId;
        _initBlock(headers);
    }

    function togglePause(bool _flag) external onlyOwner returns (bool) {
        if (_flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function setMptVerify(address _newMptVerify) external onlyOwner {
        require(_newMptVerify.code.length > 0, "_newMptVerify must contract address");
        mptVerify = _newMptVerify;
        emit SetMptVerify(_newMptVerify);
    }

    function updateBlockHeader(bytes memory _blockHeadersBytes) external override whenNotPaused {
        UpdateHeader memory updateHeader = abi.decode(_blockHeadersBytes, (UpdateHeader));
        require(_lastSyncedBlock != 0, "light node uninitialized");
        _lastSyncedBlock += EPOCH_NUM;
        BlockHeader memory header = _checkUpdateHeader(updateHeader);
        require(header.number == _lastSyncedBlock, "invalid syncing block");
        uint256 index = header.number / EPOCH_NUM;
        BLSPublicKeys[index] = Helper._getBLSPublicKey(header.extraData);
        _removeExcessEpochValidators();
        emit UpdateBlockHeader(tx.origin, header.number);
    }

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        BlockHeader memory header = _checkUpdateHeader(proof.updateHeader);
        return _verifyProofData(proof.receiptProof, header.receiptsRoot);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external override returns (bool success, string memory message, bytes memory logs) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        uint256 headerLen = proof.updateHeader.headers.length;
        uint256 verifyBlockNum = proof.updateHeader.headers[headerLen - 1].number;
        bytes32 receiptRoot = cachedReceiptRoot[verifyBlockNum];
        if (cachedReceiptRoot[verifyBlockNum] != bytes32("")) {
            (success, logs) = Helper._validateProof(receiptRoot, proof.receiptProof, mptVerify);
            if (!success) {
                message = "mpt verification failed";
            }
        } else {
            BlockHeader memory header = _checkUpdateHeader(proof.updateHeader);
            (success, message, logs) = _verifyProofData(proof.receiptProof, header.receiptsRoot);
            if (success) cachedReceiptRoot[verifyBlockNum] = header.receiptsRoot;
        }
    }

    function _verifyProofData(
        ReceiptProof memory receiptProof,
        bytes32 root
    ) private view returns (bool success, string memory message, bytes memory logs) { 
        (success, logs) = Helper._validateProof(root, receiptProof, mptVerify);
        if (!success) {
            message = "mpt verification failed";
        }
    }


    function _checkUpdateHeader(UpdateHeader memory _updateHeader) internal view returns(BlockHeader memory header){
        BlockHeader[] memory headers = _updateHeader.headers;
        uint256 headerLen = headers.length;
        header = headers[headerLen - 1];
        Helper._checkUpdateHeader(_updateHeader);
        VoteAttestation[2] memory voteAttestations = _updateHeader.voteAttestations;
        for (uint i = 0; i < 2; i++) {
            bytes[] memory _BLSPublicKeys = _getBLSPublicKeysByNumber(voteAttestations[i].Data.TargetNumber);
            Helper._verifyVoteAttestation(voteAttestations[i], _BLSPublicKeys);
        }
    }

    function _initBlock(BlockHeader[2] calldata _headers) internal {
        require(_lastSyncedBlock == 0, "already init");

        require(_headers[0].number + EPOCH_NUM == _headers[1].number);

        for (uint256 i = 0; i < 2; i++) {
            require(_headers[i].number % EPOCH_NUM == 0, "invalid init block number");
            uint256 index = _headers[i].number / EPOCH_NUM;
            BLSPublicKeys[index] = Helper._getBLSPublicKey(_headers[i].extraData);
        }
        minValidBlocknum = _headers[1].number;
        _lastSyncedBlock = _headers[1].number;
    }

    function _getBLSPublicKeysByNumber(uint256 _blockNum) internal view returns(bytes[] memory){
        uint256 index = (_blockNum / EPOCH_NUM) - 1;
        return BLSPublicKeys[index];
    }

    function _removeExcessEpochValidators() internal {
        if (_lastSyncedBlock > EPOCH_NUM * MAX_SAVED_EPOCH_NUM) {
            uint256 remove = _lastSyncedBlock - EPOCH_NUM * MAX_SAVED_EPOCH_NUM;
            if(remove >= (minValidBlocknum - EPOCH_NUM)) {
                uint256 index = remove / EPOCH_NUM;
                delete BLSPublicKeys[index];
                minValidBlocknum += EPOCH_NUM;
            }
        }
    }

    function getBytes(ProofData calldata _proof) external pure returns (bytes memory) {
        return abi.encode(_proof);
    }

    function getHeadersBytes(UpdateHeader memory _updateHeader) external pure returns (bytes memory) {
        return abi.encode(_updateHeader);
    }

    // function getBlockHash(BlockHeader memory _header) external pure returns (bytes32) {

    //     return Helper._getBlockHash(_header);
    // }

    // function getVoteDataRlpHash(VoteData memory _data) external pure returns (bytes32) {

    //     return Helper._getVoteDataRlpHash(_data);
    // }

    function headerHeight() external view override returns (uint256) {
        return _lastSyncedBlock;
    }

    function maxCanVerifyNum() public view returns (uint256) {
        return _lastSyncedBlock + EPOCH_NUM + EPOCH_NUM - 1;
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
