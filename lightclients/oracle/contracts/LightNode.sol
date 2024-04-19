// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "./lib/Verify.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    address public mptVerify;

    mapping(uint256 => bytes32) public receiptRoots;

    address public oracle;

    uint256 public chainId;

    address private _pendingAdmin;

    uint256 private _nodeType;

    event SetMptVerify(address newMptVerify);
    event SetOracle(address _oracle);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event AdminTransferred(address indexed previous, address indexed newAdmin);

    struct ProofData {
        uint256 blockNum;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    modifier onlyOracle() {
        require(msg.sender == oracle, "lightnode :: only oracle");
        _;
    }

    constructor() {}

    function initialize(uint256 _chainId, address _controller, address _mptVerify, uint256 _node) external initializer {
        require(_chainId > 0, "invalid _chainId");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        chainId = _chainId;
        mptVerify = _mptVerify;
        _nodeType = _node;
        _changeAdmin(_controller);
    }

    function removeRoot(uint256 _blockNumber) external onlyOwner {
        receiptRoots[_blockNumber] = bytes32("");
    }

    function setMptVerify(address _verifier) external onlyOwner {
        require(_verifier != address(0), "LightNode: verifier is the zero address");
        mptVerify = _verifier;
        emit SetMptVerify(_verifier);
    }

    function setOracle(address _oracle) external onlyOwner {
        require(_oracle != address(0), "LightNode: _oracle is the zero address");
        oracle = _oracle;
        emit SetOracle(_oracle);
    }

    function togglePause(bool _flag) external onlyOwner returns (bool) {
        if (_flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function updateBlockHeader(bytes memory _blockHeadersBytes) external override whenNotPaused onlyOracle {
        (uint256 blockNum, bytes32 receiptRoot) = abi.decode(_blockHeadersBytes, (uint256, bytes32));
        require(blockNum > 0, "LightNode: zero block number");
        require(receiptRoot != bytes32(""), "LightNode: empty receipt root");
        require(receiptRoots[blockNum] == bytes32(""), "LightNode: already update");
        receiptRoots[blockNum] = receiptRoot;
        emit UpdateBlockHeader(tx.origin, blockNum);
    }

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        return _verifyProofData(_receiptProof);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        return _verifyProofData(_receiptProof);
    }

    function _verifyProofData(
        bytes memory _receiptProof
    ) private view returns (bool success, string memory message, bytes memory logs) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        bytes32 rootHash = receiptRoots[proof.blockNum];
        require(rootHash != bytes32(""), "LightNode: receipt root not update");
        (success, logs) = Verify._validateProof(rootHash, proof.receiptProof, mptVerify);
        if (!success) {
            message = "mpt verification failed";
        }
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        return receiptRoots[_blockHeight] != bytes32("");
    }

    function nodeType() external view override returns (uint256) {
        // return this chain light node type on target chain
        // 1 default light client
        // 2 zk light client
        // 3 oracle client
        return _nodeType;
    }

    function notifyLightClient(address _from, bytes memory _data) external override {
        emit ClientNotifySend(_from, block.number, _data);
    }

    function getBytes(ProofData calldata _proof) external pure returns (bytes memory) {
        return abi.encode(_proof);
    }

    function getHeadersBytes(uint256 blockNum, bytes32 receiptRoot) external pure returns (bytes memory) {
        return abi.encode(blockNum, receiptRoot);
    }

    function headerHeight() external view override returns (uint256) {
        return 0;
    }

    function verifiableHeaderRange() external view override returns (uint256, uint256) {
        return (0, 0);
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
