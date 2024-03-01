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
    address public lightNode;
    uint256 public quorum;
    uint256 public proposerCount;

    mapping(address => bool) public proposers;

    // blockNum => receiptRoot => proposeCount
    mapping(uint256 => mapping(bytes32 => uint256)) public proposals;
    // blockNum => proposer => receiptRoot
    mapping(uint256 => mapping(address => bytes32)) public records;

    event SetQuorum(uint256 indexed _quorum);
    event SetLightNode(address indexed _lightNode);
    event UpdateProposer(address indexed _approver, bool indexed _flag);
    event Execute(uint256 indexed _blockNum, bytes32 indexed _receiptRoot);
    event Propose(address indexed proposer, uint256 indexed blockNum, bytes32 indexed receiptRoot);
    event RecoverProposal(address indexed proposer, uint256 indexed blockNum);

    modifier onlyProposer() {
        require(proposers[msg.sender], "oracle: only proposer");
        _;
    }

    modifier quorumSet() {
        require(quorum != 0, "oracle: quorum not set");
        _;
    }

    constructor(address _owner) {
        _transferOwnership(_owner);
    }

    function setLightNode(address _lightNode) external onlyOwner {
        require(_lightNode != address(0), "oracle: address_0");
        lightNode = _lightNode;
        emit SetLightNode(_lightNode);
    }

    // pause before update proposer
    function setQuorum(uint256 _quorum) external onlyOwner {
        require(_quorum != 0, "oracle: value_0");
        require(_quorum <= proposerCount, "oracle: quorum gt proposerCount");
        quorum = _quorum;
        emit SetQuorum(_quorum);
    }

    // pause before update proposer
    function updateProposer(address[] memory _proposers, bool _flag) external onlyOwner {
        for (uint i = 0; i < _proposers.length; i++) {
            require(_proposers[i] != address(0), "oracle: address_0");
            require(proposers[_proposers[i]] != _flag, "oracle: already is");
            proposers[_proposers[i]] = _flag;
            if (_flag) proposerCount++;
            else proposerCount--;
            emit UpdateProposer(_proposers[i], _flag);
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

    function propose(uint256 blockNum, bytes32 receiptRoot) external whenNotPaused onlyProposer quorumSet {
        address proposer = msg.sender;
        require(blockNum != 0, "oracle: value_0");
        require(receiptRoot != bytes32(""), "oracle: empty receipt root");
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum);
        require(r == bytes32(""), "oracle: already update");
        require(records[blockNum][proposer] == bytes32(""), "oracle: proposer already propose this blockNum");

        records[blockNum][proposer] = receiptRoot;
        proposals[blockNum][receiptRoot]++;
        if (proposals[blockNum][receiptRoot] >= quorum) {
            _execute(blockNum, receiptRoot);
        }
        emit Propose(proposer, blockNum, receiptRoot);
    }

    function execute(uint256 blockNum, bytes32 receiptRoot) external {
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum);
        require(r == bytes32(""), "already update");
        require(proposals[blockNum][receiptRoot] >= quorum, "oracle: approve not enough");
        _execute(blockNum, receiptRoot);
    }

    function _execute(uint256 _blockNum, bytes32 _receiptRoot) private {
        bytes memory u = abi.encode(_blockNum, _receiptRoot);
        IOracleLightNode(lightNode).updateBlockHeader(u);
        emit Execute(_blockNum, _receiptRoot);
    }

    function recoverPropose(address proposer, uint256 blockNum) external onlyOwner {
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum);
        require(r == bytes32(""), "already update");
        require(records[blockNum][proposer] != bytes32(""), "oracle: proposer not propose this blockNum");
        delete records[blockNum][proposer];
        bytes32 receiptRoot = records[blockNum][proposer];
        proposals[blockNum][receiptRoot]--;
        emit RecoverProposal(proposer, blockNum);
    }
}
