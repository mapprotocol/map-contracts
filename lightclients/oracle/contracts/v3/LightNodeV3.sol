// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/access/AccessControlEnumerable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "../abstract/ECDSAMultisig.sol";


contract LightNodeV3 is ECDSAMultisig, UUPSUpgradeable, Initializable, Pausable, AccessControlEnumerable, ILightNode {
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    // the reserved high 128 bits of position carry the signature expiry (unix timestamp in seconds)
    uint256 private constant EXPIRY_SHIFT = 128;

    uint256 public chainId;
    
    uint256 private _nodeType;

    error LightNodeV3_Unsupport_Type();
    error LightNodeV3_Invalid_Log_Bytes();
    error LightNodeV3_Invalid_Position();
    error LightNodeV3_Signature_Expired(uint256 expiry, uint256 currentTime);

    event UpdateMultisig(bytes32 version, uint256 quorum, address[] signers);

    enum ProofType { MPT, LOG }

    struct ProofData {
        ProofType proofType;
        //position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
        //expiry: signature is valid while block.timestamp <= expiry, 0 means it never expires
        uint256 position;
        bytes32 receiptRoot;
        bytes[] signatures;
        bytes encodeLog;
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(uint256 _chainId, address _defaultAdmin) external initializer {
        require(_defaultAdmin != address(0));
        _grantRole(MANAGER_ROLE, _defaultAdmin);
        _grantRole(UPGRADER_ROLE, _defaultAdmin);
        _grantRole(PAUSER_ROLE, _defaultAdmin);
        _grantRole(DEFAULT_ADMIN_ROLE, _defaultAdmin);

        require(_chainId > 0, "invalid _chainId");
        chainId = _chainId;
        _nodeType = 5;
    }

    function updateMultisig(uint256 quorum, address[] calldata signers) external onlyRole(MANAGER_ROLE) {
        if(quorum == 0) revert ECDSAMultisig_QuorumValueZero();
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

    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(MANAGER_ROLE) {
        _unpause();
    }

    function updateBlockHeader(bytes memory _blockHeader) external override {}

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override whenNotPaused returns (bool success, string memory message, bytes memory logs) {
        return _verifyProofData(_receiptProof);
    }

    function verifyProofData(
        uint256,
        bytes memory _receiptProof
    ) external view override whenNotPaused returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        return _verifyProofDataAndDecodeLog(_receiptProof);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external view override whenNotPaused returns (bool success, string memory message, bytes memory logs) {
        return _verifyProofData(_receiptProof);
    }

    function verifyProofDataWithCache(
        bool,
        uint256,
        bytes memory _receiptProof
    ) external view  override whenNotPaused returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        return _verifyProofDataAndDecodeLog(_receiptProof);
    }

    function _verifyProofDataAndDecodeLog(
        bytes memory _receiptProof
    ) private view returns (bool success, string memory message, ILightVerifier.txLog memory log) {
        bytes memory logBytes;
        (success, message, logBytes) = _verifyProofData(_receiptProof);
        if(success) log = _decodeLogFromBytes(logBytes);
    }

    function _verifyProofData(
        bytes memory _receiptProof
    ) private view returns (bool success, string memory message, bytes memory logs) {
        ProofData memory data = abi.decode(_receiptProof, (ProofData));
        _validatePosition(data.position);
        _validateExpiry(data.position);
        _verifySignatures(data.receiptRoot, data.position, chainId, data.signatures);
        if(data.proofType != ProofType.LOG) revert LightNodeV3_Unsupport_Type();
        if(keccak256(data.encodeLog) == data.receiptRoot){
            success = true;
            logs = data.encodeLog;
        } else {
            success = false;
            message = "invalid event bytes";
        }
        
        
    }

    function _validatePosition(uint256 position) private pure {
        // the low 128 bits (log index | tx index | block number) must not be empty
        if (uint128(position) == 0) revert LightNodeV3_Invalid_Position();
    }

    function _validateExpiry(uint256 position) private view {
        uint256 expiry = position >> EXPIRY_SHIFT;
        if (expiry != 0 && block.timestamp > expiry) revert LightNodeV3_Signature_Expired(expiry, block.timestamp);
    }

    //addr(20) + 4 + 4(topic num) +4(data len) + topic[] + data
    function _decodeLogFromBytes(bytes memory _logBytes) internal pure returns(ILightVerifier.txLog memory log){
        if (_logBytes.length < 32) revert LightNodeV3_Invalid_Log_Bytes();
        address addr;
        uint256 topicNum;
        uint256 dataLen;
        uint256 point;
        assembly {
            //skip 32 byte data length
            point := add(_logBytes, 32)
            let firstWord := mload(point)
            addr := shr(96, firstWord)
            topicNum := shr(32, and(firstWord, 0x000000000000000000000000000000000000000000000000ffffffff00000000))
            dataLen := and(firstWord, 0x00000000000000000000000000000000000000000000000000000000ffffffff)
        }
        uint256 expectedLength = 32 + topicNum * 32 + dataLen;
        if (_logBytes.length != expectedLength) revert LightNodeV3_Invalid_Log_Bytes();
        log.addr = addr;
        log.topics = new bytes32[](topicNum);

        for(uint256 i = 0; i < topicNum; i++) {
            point += 32;
            bytes32 t;
            assembly {
               t := mload(point)
            }
            log.topics[i] = t;
        }

        bytes memory d;
        assembly {
            mstore(point, dataLen)
            d := point
        }
        log.data = d;
    }



    function multisigInfo() external view returns (bytes32 version, uint256 quorum, address[] memory singers) {
        return _multisigInfo();
    }

    function isVerifiable(uint256, bytes32) external view override returns (bool) {
        return (_quorum() != 0 && !paused());
    }

    function nodeType() external view override returns (uint256) {
        // return this chain light node type on target chain
        // 1 default light client
        // 2 zk light client
        // 3 oracle node v1
        // 4 oracle node v2 - mpt verification
        // 5 oracle node v2 - log verification
        return _nodeType;
    }

    function notifyLightClient(address _from, bytes memory _data) external override {
        emit ClientNotifySend(_from, block.number, _data);
    }

    function getBytes(ProofData calldata _proof) external pure returns (bytes memory) {
        return abi.encode(_proof);
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
        require(hasRole(UPGRADER_ROLE, msg.sender), "only upgrade role");
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
