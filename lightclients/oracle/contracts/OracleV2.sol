// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "./abstract/ECDSAMultisig.sol";

contract OracleV2 is ECDSAMultisig, Ownable, Pausable, ReentrancyGuard {
    struct LightNodeInfo {
        // uint256(hash(version,blockNum)) => receiptRoot => signature
        mapping(uint256 => mapping(bytes32 => bytes[])) proposals;
        // uint256(hash(version,blockNum)) => signer => receiptRoot
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

    event UpdateMultisig(bytes32 version, uint256 quorum, address[] signers);
    event Meet(uint256 chainId, uint256 blockNum, bytes32 rootHash, bytes[] signature);
    event RecoverProposal(uint256 chainId, uint256 blockNum, address signer, uint256 index);
    event Proposal(address signer, uint256 chainId, uint256 blockNum, bytes32 rootHash, bytes signature);

    constructor(address _owner) {
        _transferOwnership(_owner);
    }

    function togglePause() external onlyOwner {
        paused() ? _unpause() : _pause();
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

    function propose(
        uint256 chainId,
        uint256 blockNum,
        bytes32 rootHash,
        bytes calldata signature
    ) external whenNotPaused nonReentrant {
        if (chainId == 0 || blockNum == 0 || rootHash == bytes32("")) revert invalid_proposal_param();
        address signer = _verifySignature(rootHash, blockNum, chainId, signature);
        if (msg.sender != signer) revert only_signer();

        bytes32 version = _version();
        uint256 key = _getKey(version, blockNum);
        LightNodeInfo storage info = infos[chainId];
        uint256 len = info.proposals[key][rootHash].length;
        uint256 quorum = _quorum();
        if (len == quorum) revert already_meet();

        bytes32 beforeProposal = info.records[key][signer];
        if (beforeProposal != bytes32("")) revert already_proposal();
        info.proposals[key][rootHash].push(abi.encode(signer, signature));
        len++;
        info.records[key][signer] = rootHash;
        emit Proposal(signer, chainId, blockNum, rootHash, signature);

        if (len == quorum) {
            bytes[] memory signatures = new bytes[](len);
            for (uint i = 0; i < len; i++) {
                (, bytes memory s) = _split(info.proposals[key][rootHash][i]);
                signatures[i] = s;
            }
            // delete info.proposals[key][rootHash];
            emit Meet(chainId, blockNum, rootHash, signatures);
        }
    }

    function recoverProposal(uint256 chainId, uint256 blockNum, address signer, uint256 index) external {
        if (msg.sender != signer && msg.sender != owner()) revert only_signer_or_owner();
        bytes32 version = _version();
        uint256 key = _getKey(version, blockNum);
        LightNodeInfo storage info = infos[chainId];
        bytes32 beforeProposal = info.records[key][signer];
        if (beforeProposal == bytes32("")) revert not_proposal();
        info.records[key][signer] = bytes32("");
        _deleteSignature(info.proposals[key][beforeProposal], signer, index);
        emit RecoverProposal(chainId, blockNum, signer, index);
    }

    function proposalInfo(
        uint256 chainId,
        uint256 blockNum,
        bytes32 rootHash,
        bytes32 version
    ) external view returns (address[] memory singers, bytes[] memory signatures, bool canVerify) {
        if (version == bytes32("")) version = _version();
        uint256 key = _getKey(version, blockNum);
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
        canVerify = (len >= _quorum());
    }

    function isProposed(
        uint256 chainId,
        bytes32 version,
        uint256 blockNum,
        address signer
    ) external view returns (bytes32) {
        uint256 key = _getKey(version, blockNum);
        LightNodeInfo storage info = infos[chainId];
        return info.records[key][signer];
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

    function _getKey(bytes32 version, uint256 blockNum) private pure returns (uint256) {
        return uint256(keccak256(abi.encodePacked(version, blockNum)));
    }
}
