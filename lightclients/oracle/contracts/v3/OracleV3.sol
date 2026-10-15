// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;


import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/access/AccessControlEnumerable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "../abstract/ECDSAMultisig.sol";

contract OracleV3 is ECDSAMultisig, UUPSUpgradeable, Initializable, AccessControlEnumerable, Pausable, ReentrancyGuard {
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    uint256 private constant POSITION_MASK = type(uint128).max;
    // the high 128 bits of position carry the signature expiry (unix timestamp in seconds, 0 = never expires)
    uint256 private constant EXPIRY_SHIFT = 128;
    uint256 private constant BLOCK_NUM_MASK = type(uint64).max;
    uint256 private constant TX_INDEX_MASK = type(uint32).max;
    uint256 private constant LOG_INDEX_MASK = type(uint32).max;

    struct LightNodeInfo {
        // uint256(hash(version,position)) => receiptRoot => signature
        mapping(uint256 => mapping(bytes32 => bytes[])) proposals;
        // uint256(hash(version,position)) => signer => receiptRoot
        mapping(uint256 => mapping(address => bytes32)) records;
    }
    // chainId => LightNodeInfo
    mapping(uint256 => LightNodeInfo) private infos;

    error already_meet();
    error only_signer();
    error not_proposal();
    error already_proposal();
    error signatures_out_bond();
    error only_signer_or_owner();
    error singer_mismatching();
    error invalid_proposal_param();
    error proposal_expired(uint256 expiry, uint256 currentTime);

    event UpdateMultisig(bytes32 version, uint256 quorum, address[] signers);
    event Meet(uint256 chainId, uint256 position, bytes32 rootHash, bytes[] signature);
    event RecoverProposal(uint256 chainId, uint256 position, address signer, uint256 index);
    event Proposal(address signer, uint256 chainId, uint256 position, bytes32 rootHash, bytes signature);

    constructor() {
        _disableInitializers();
    }

    function initialize(address _defaultAdmin) public initializer {
        require(_defaultAdmin != address(0));
        _grantRole(MANAGER_ROLE, _defaultAdmin);
        _grantRole(UPGRADER_ROLE, _defaultAdmin);
        _grantRole(PAUSER_ROLE, _defaultAdmin);
        _grantRole(DEFAULT_ADMIN_ROLE, _defaultAdmin);
    }

    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(MANAGER_ROLE) {
        _unpause();
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

    // position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
    function propose(
        uint256 chainId,
        uint256 position,
        bytes32 rootHash,
        bytes calldata signature
    ) external whenNotPaused nonReentrant {
       _validatePosition(position);
       _propose(chainId, position, rootHash, signature);
    }

    function propose(
        uint256 chainId,
        uint256 blockNum,
        uint256 txIndex,
        uint256 logIndex,
        uint256 expiry,
        bytes32 rootHash,
        bytes calldata signature
    ) external whenNotPaused nonReentrant {
       _propose(chainId, _encodePosition(blockNum, txIndex, logIndex, expiry), rootHash, signature);
    }

    struct BatchProposeParam {
        uint256 chainId;
        uint256 position;
        bytes32 rootHash;
        bytes  signature;
    }

    function batchPropose(
       BatchProposeParam[] calldata params
    ) external whenNotPaused nonReentrant {
       uint256 len = params.length;
       for (uint i = 0; i < len; i++) {
            _validatePosition(params[i].position);
            _propose(params[i].chainId, params[i].position, params[i].rootHash, params[i].signature);
       }
    }

    function _propose(
        uint256 chainId,
        uint256 position,
        bytes32 rootHash,
        bytes calldata signature
    ) private {
        if (chainId == 0 || rootHash == bytes32("")) revert invalid_proposal_param();
        _validateExpiry(position);
        address signer = _verifySignature(rootHash, position, chainId, signature);
        if (msg.sender != signer) revert only_signer();

        bytes32 version = _version();
        uint256 key = _getKey(version, position);
        LightNodeInfo storage info = infos[chainId];
        uint256 len = info.proposals[key][rootHash].length;
        uint256 quorum = _quorum();
        if (len == quorum) revert already_meet();

        bytes32 beforeProposal = info.records[key][signer];
        if (beforeProposal != bytes32("")) revert already_proposal();
        info.proposals[key][rootHash].push(abi.encode(signer, signature));
        len++;
        info.records[key][signer] = rootHash;
        emit Proposal(signer, chainId, position, rootHash, signature);

        if (len == quorum) {
            bytes[] memory signatures = new bytes[](len);
            for (uint i = 0; i < len; i++) {
                (, bytes memory s) = _split(info.proposals[key][rootHash][i]);
                signatures[i] = s;
            }
            // delete info.proposals[key][rootHash];
            emit Meet(chainId, position, rootHash, signatures);
        }
    }
    
    // position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
    function recoverProposal(uint256 chainId, uint256 position, address signer, uint256 index) external {
        _validatePosition(position);
        _recoverProposal(chainId, position, signer, index);
    }

    function recoverProposal(
        uint256 chainId,
        uint256 blockNum,
        uint256 txIndex,
        uint256 logIndex,
        uint256 expiry,
        address signer,
        uint256 index
    ) external {
        _recoverProposal(chainId, _encodePosition(blockNum, txIndex, logIndex, expiry), signer, index);
    }

    // position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
    function proposalInfo(
        uint256 chainId,
        uint256 position,
        bytes32 rootHash,
        bytes32 version
    ) external view returns (address[] memory singers, bytes[] memory signatures, bool canVerify) {
       _validatePosition(position);
       return _proposalInfo(chainId, position, rootHash, version);
    }

    function proposalInfo(
        uint256 chainId,
        uint256 blockNum,
        uint256 txIndex,
        uint256 logIndex,
        uint256 expiry,
        bytes32 rootHash,
        bytes32 version
    ) external view returns (address[] memory singers, bytes[] memory signatures, bool canVerify) {
        return _proposalInfo(chainId, _encodePosition(blockNum, txIndex, logIndex, expiry), rootHash, version);
    }

    function _proposalInfo(
        uint256 chainId,
        uint256 position,
        bytes32 rootHash,
        bytes32 version
    ) internal view returns (address[] memory singers, bytes[] memory signatures, bool canVerify) {
        if (version == bytes32("")) version = _version();
        uint256 key = _getKey(version, position);
        LightNodeInfo storage info = infos[chainId];
        uint256 len = info.proposals[key][rootHash].length;
        singers = new address[](len);
        signatures = new bytes[](len);
        for (uint i = 0; i < len; i++) {
            address signer;
            bytes memory signature;
            (signer, signature) = _split(info.proposals[key][rootHash][i]);
            singers[i] = signer;
            signatures[i] = signature;
        }
        canVerify = (_version() == version && len >= _quorum());
    }

    // position: (expiry 16 bytes | log index 4 bytes | tx index 4 bytes | block number 8 bytes)
    function isProposed(
        uint256 chainId,
        bytes32 version,
        uint256 position,
        address signer
    ) external view returns (bytes32) {
        _validatePosition(position);
        return _isProposed(chainId, version, position, signer);
    }

    function isProposed(
        uint256 chainId,
        bytes32 version,
        uint256 blockNum,
        uint256 txIndex,
        uint256 logIndex,
        uint256 expiry,
        address signer
    ) external view returns (bytes32) {
        uint256 position = _encodePosition(blockNum, txIndex, logIndex, expiry);
        return _isProposed(chainId, version, position, signer);
    }

    function _isProposed(
        uint256 chainId,
        bytes32 version,
        uint256 position,
        address signer
    ) internal view returns (bytes32) {
        uint256 key = _getKey(version, position);
        LightNodeInfo storage info = infos[chainId];
        return info.records[key][signer];
    }

    // expiry: unix timestamp in seconds, the signature is valid while block.timestamp <= expiry, 0 never expires
    function encodePosition(
        uint256 blockNum,
        uint256 txIndex,
        uint256 logIndex,
        uint256 expiry
    ) external pure returns (uint256) {
        return _encodePosition(blockNum, txIndex, logIndex, expiry);
    }

    function decodePosition(
        uint256 position
    ) external pure returns (uint256 blockNum, uint256 txIndex, uint256 logIndex, uint256 expiry) {
        _validatePosition(position);
        return _decodePosition(position);
    }

    function multisigInfo() external view returns (bytes32 version, uint256 quorum, address[] memory singers) {
        return _multisigInfo();
    }

    function _deleteSignature(bytes[] storage signatures, address singer, uint256 index) private {
        uint256 len = signatures.length;
        if (len <= index) revert signatures_out_bond();
        (address s, ) = _split(signatures[index]);
        if (s != singer) revert singer_mismatching();
        bytes memory last = signatures[len - 1];
        signatures[index] = last;
        signatures.pop();
    }

    function _split(bytes memory data) private pure returns (address signer, bytes memory signature) {
        (signer, signature) = abi.decode(data, (address, bytes));
    }

    function _recoverProposal(uint256 chainId, uint256 position, address signer, uint256 index) private {
        if (msg.sender != signer && !hasRole(MANAGER_ROLE, msg.sender)) revert only_signer_or_owner();
        bytes32 version = _version();
        uint256 key = _getKey(version, position);
        LightNodeInfo storage info = infos[chainId];
        bytes32 beforeProposal = info.records[key][signer];
        if (beforeProposal == bytes32("")) revert not_proposal();
        info.records[key][signer] = bytes32("");
        _deleteSignature(info.proposals[key][beforeProposal], signer, index);
        emit RecoverProposal(chainId, position, signer, index);
    }

    function _getKey(bytes32 version, uint256 position) private pure returns (uint256) {
        return uint256(keccak256(abi.encodePacked(version, position)));
    }

    function _validatePosition(uint256 position) private pure {
        // the low 128 bits (log index | tx index | block number) must not be empty, the high
        // 128 bits are the signature expiry that LightNodeV3 enforces when it verifies the proof
        if (position & POSITION_MASK == 0) revert invalid_proposal_param();
    }

    // only new proposals are checked, recovering and reading old proposals stays possible after the expiry
    function _validateExpiry(uint256 position) private view {
        uint256 expiry = position >> EXPIRY_SHIFT;
        if (expiry != 0 && block.timestamp > expiry) revert proposal_expired(expiry, block.timestamp);
    }

    function _encodePosition(
        uint256 blockNum,
        uint256 txIndex,
        uint256 logIndex,
        uint256 expiry
    ) private pure returns (uint256 position) {
        if (
            blockNum == 0 ||
            blockNum > BLOCK_NUM_MASK ||
            txIndex > TX_INDEX_MASK ||
            logIndex > LOG_INDEX_MASK ||
            expiry > POSITION_MASK
        ) revert invalid_proposal_param();
        return (expiry << EXPIRY_SHIFT) | (logIndex << 96) | (txIndex << 64) | blockNum;
    }

    function _decodePosition(
        uint256 position
    ) private pure returns (uint256 blockNum, uint256 txIndex, uint256 logIndex, uint256 expiry) {
        _validatePosition(position);
        blockNum = position & BLOCK_NUM_MASK;
        txIndex = (position >> 64) & TX_INDEX_MASK;
        logIndex = (position >> 96) & LOG_INDEX_MASK;
        expiry = position >> EXPIRY_SHIFT;
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(hasRole(UPGRADER_ROLE, msg.sender), "only upgrade role");
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
