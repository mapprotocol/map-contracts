// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";

interface IOracleLightNode is ILightNode {
    function receiptRoots(uint256 _blockNum) external returns (bytes32);
}

contract Oracle is Ownable, Pausable, ReentrancyGuard {
    struct LightNodeInfo {
        address lightNode;
        uint256 quorum;
        uint256 proposerCount;
        mapping(address => bool) proposers;
        // blockNum => receiptRoot => proposeCount
        mapping(uint256 => mapping(bytes32 => uint256)) proposals;
        // blockNum => proposer => receiptRoot
        mapping(uint256 => mapping(address => bytes32)) records;
    }
    // chainId => LightNodeInfo
    mapping(uint256 => LightNodeInfo) private infos;

    event SetQuorum(uint256 indexed _chainId, uint256 indexed _quorum);
    event SetLightNode(uint256 indexed _chainId, address indexed _lightNode);
    event UpdateProposer(uint256 indexed _chainId, address indexed _proposer, bool indexed _flag);
    event Execute(uint256 indexed _chainId, uint256 indexed _blockNum, bytes32 indexed _receiptRoot);
    event UpdateBlockHeader(
        uint256 indexed _chainId,
        address indexed proposer,
        uint256 indexed blockNum,
        bytes32 receiptRoot
    );
    event RecoverProposal(uint256 indexed _chainId, address indexed proposer, uint256 indexed blockNum);

    constructor(address _owner) {
        _transferOwnership(_owner);
    }

    function setLightNode(uint256 _chainId, address _lightNode) external onlyOwner {
        // require(_lightNode != address(0), "oracle: address_0");
        infos[_chainId].lightNode = _lightNode;
        emit SetLightNode(_chainId, _lightNode);
    }

    // pause before update proposer
    function setQuorum(uint256 _chainId, uint256 _quorum) external onlyOwner {
        require(_quorum != 0, "oracle: value_0");
        LightNodeInfo storage i = infos[_chainId];
        require(_quorum <= i.proposerCount, "oracle: quorum gt proposerCount");
        i.quorum = _quorum;
        emit SetQuorum(_chainId, _quorum);
    }

    // pause before update proposer
    function updateProposer(uint256 _chainId, address[] memory _proposers, bool _flag) external onlyOwner {
        LightNodeInfo storage info = infos[_chainId];
        for (uint i = 0; i < _proposers.length; i++) {
            address proposer = _proposers[i];
            require(proposer != address(0), "oracle: address_0");
            require(info.proposers[proposer] != _flag, "oracle: already set");
            info.proposers[proposer] = _flag;
            if (_flag) info.proposerCount++;
            else info.proposerCount--;
            emit UpdateProposer(_chainId, proposer, _flag);
        }
    }

    function togglePause(bool _flag) external onlyOwner returns (bool) {
        if (_flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function updateBlockHeader(uint256 _chainId, bytes memory _headerBytes) external whenNotPaused {
        require(_headerBytes.length != 64, "oracle: invalid bytes");
        (uint256 blockNum, bytes32 receiptRoot) = abi.decode(_headerBytes, (uint256, bytes32));
        address proposer = msg.sender;
        require(blockNum != 0, "oracle: value_0");
        require(receiptRoot != bytes32(""), "oracle: empty receipt root");
        LightNodeInfo storage info = infos[_chainId];
        require(info.lightNode != address(0), "oracle: not support chain");
        require(info.proposers[proposer], "oracle: only proposer");
        require(info.quorum != 0, "oracle: quorum not set");
        bytes32 r = IOracleLightNode(info.lightNode).receiptRoots(blockNum);
        require(r == bytes32(""), "oracle: already update");
        //
        if (info.records[blockNum][proposer] == bytes32("")) {
            info.records[blockNum][proposer] = receiptRoot;
            info.proposals[blockNum][receiptRoot]++;
        } else {
            require(info.records[blockNum][proposer] == receiptRoot, "oracle: different propose");
        }

        if (info.proposals[blockNum][receiptRoot] >= info.quorum) {
            _execute(_chainId, info.lightNode, blockNum, receiptRoot);
        }
        emit UpdateBlockHeader(_chainId, proposer, blockNum, receiptRoot);
    }

    function execute(uint256 chainId, uint256 blockNum, bytes32 receiptRoot) external {
        LightNodeInfo storage info = infos[chainId];
        bytes32 r = IOracleLightNode(info.lightNode).receiptRoots(blockNum);
        require(r == bytes32(""), "already update");
        require(info.proposals[blockNum][receiptRoot] >= info.quorum, "oracle: approve not enough");
        _execute(chainId, info.lightNode, blockNum, receiptRoot);
    }

    function recoverPropose(uint256 _chainId, address proposer, uint256 blockNum) external onlyOwner {
        LightNodeInfo storage info = infos[_chainId];
        bytes32 r = IOracleLightNode(info.lightNode).receiptRoots(blockNum);
        require(r == bytes32(""), "already update");
        bytes32 receiptRoot = info.records[blockNum][proposer];
        require(receiptRoot != bytes32(""), "oracle: proposer not propose this blockNum");
        delete info.records[blockNum][proposer];
        info.proposals[blockNum][receiptRoot]--;
        emit RecoverProposal(_chainId, proposer, blockNum);
    }

    function lightNodeInfo(
        uint256 _chainId
    ) external view returns (address lightNode, uint256 quorum, uint256 proposerCount) {
        LightNodeInfo storage i = infos[_chainId];
        lightNode = i.lightNode;
        quorum = i.quorum;
        proposerCount = i.proposerCount;
    }

    function isProposer(uint256 _chainId, address _proposer) external view returns (bool result) {
        return infos[_chainId].proposers[_proposer];
    }

    function isProposed(uint256 _chainId, address _proposer, uint256 _blockNum) external view returns (bool result) {
        LightNodeInfo storage i = infos[_chainId];
        return i.records[_blockNum][_proposer] != bytes32("");
    }

    function _execute(uint256 _chainId, address _lightNode, uint256 _blockNum, bytes32 _receiptRoot) private {
        bytes memory u = abi.encode(_blockNum, _receiptRoot);
        IOracleLightNode(_lightNode).updateBlockHeader(u);
        emit Execute(_chainId, _blockNum, _receiptRoot);
    }
}
