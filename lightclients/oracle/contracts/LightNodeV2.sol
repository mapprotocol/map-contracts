// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "./abstract/ECDSAMultisig.sol";
import "./lib/Verify.sol";

contract LightNodeV2 is ECDSAMultisig, UUPSUpgradeable, Initializable, Pausable, ILightNode {
    address public mptVerify;

    uint256 public chainId;

    address private _pendingAdmin;

    uint256 private _nodeType;

    event SetMptVerify(address newMptVerify);
    event UpdateMultisig(bytes32 version, uint256 quorum, address[] signers);
    event AdminTransferred(address indexed previous, address indexed newAdmin);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);

    struct ProofData {
        uint256 blockNum;
        bytes32 receiptRoot;
        bytes[] signatures;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
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

    function updateMultisig(uint256 quorum, address[] calldata signers) external onlyOwner {
        _setQuorum(0);
        address[] memory preSigners = _signers();
        uint256 preLen = preSigners.length;
        for (uint i = 0; i < preLen; i++) {
            _removeSigner(preSigners[i]);
        }

        uint256 len = signers.length;
        for (uint i = 0; i < len; i++) {
            _addSigner(signers[i]);
        }
        _setQuorum(quorum);

        bytes32 version = keccak256(abi.encodePacked(quorum, signers));
        _setVersion(version);
        emit UpdateMultisig(version, quorum, signers);
    }

    /*
    function setMptVerify(address _verifier) external onlyOwner {
        require(_verifier != address(0), "LightNode: verifier is the zero address");
        mptVerify = _verifier;
        emit SetMptVerify(_verifier);
    }*/

    function togglePause() external onlyOwner {
         paused() ? _unpause() : _pause();
    }

    function updateBlockHeader(bytes memory _blockHeader) external override {}

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override whenNotPaused returns (bool success, string memory message, bytes memory logs) {
        return _verifyProofData(_receiptProof);
    }

    function verifyProofData(
        uint256 _logIndex,
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        return _verifyProofData(_logIndex, _receiptProof);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external override whenNotPaused returns (bool success, string memory message, bytes memory logs) {
        return _verifyProofData(_receiptProof);
    }


    function verifyProofDataWithCache(
        bool _cache,
        uint256 _logIndex,
        bytes memory _receiptProof
    ) external override returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        return _verifyProofData(_logIndex, _receiptProof);
    }


    function _verifyProofData(
        bytes memory _receiptProof
    ) private view returns (bool success, string memory message, bytes memory logs) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        _verifySignatures(proof.receiptRoot, proof.blockNum, chainId, proof.signatures);
        (success, logs) = Verify._validateProof(proof.receiptRoot, proof.receiptProof, address(0x0));
        require(success, "mpt verification failed");
    }

    function _verifyProofData(
        uint256 _logIndex,
        bytes memory _receiptProof
    ) private view returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));
        _verifySignatures(proof.receiptRoot, proof.blockNum, chainId, proof.signatures);
        (success, log) = Verify._validateProofWithLog(_logIndex, proof.receiptRoot, proof.receiptProof);

        require(success, "mpt verification failed");
    }

    function multisigInfo() external view returns (bytes32 version, uint256 quorum, address[] memory singers) {
        return _multisigInfo();
    }

    function isVerifiable(uint256, bytes32) external view override returns (bool) {
        return true;
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

    /*
    function getHeadersBytes(uint256 blockNum, bytes32 receiptRoot) external pure returns (bytes memory) {
        return abi.encode(blockNum, receiptRoot);
    }*/

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
