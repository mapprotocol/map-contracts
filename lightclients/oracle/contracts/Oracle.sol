// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";

interface IOracleLightNode is ILightNode {
   function receiptRoots(uint256 _blockNum) external  returns (bytes32);
}

contract Oracle is Ownable,ReentrancyGuard{
    address public lightNode;
    uint256 public quorum;
    uint256 public proposerCount;

    mapping(address => bool) public proposers;

    //receiptRoot => blockNum => proposeCount
    mapping (bytes32 => mapping(uint256 => uint256)) public proposes;
     //proposer => blockNum => receiptRoot
    mapping (address => mapping(uint256 => bytes32)) public records;

    event SetQuorum(uint256 indexed _quorum);
    event SetLightNode(address indexed _lightNode);
    event UpdateProposer(address indexed _approver,bool indexed _flag);
    event Excute(uint256 indexed _blockNum,bytes32 indexed _receiptRoot);
    event ProposeEvent(address indexed proposers, uint256 indexed blockNum,bytes32 indexed receiptRoot);
    event RecoverPropose(address indexed proposer,uint256 indexed blockNum);

    modifier  onlyProposer {
        require(proposers[msg.sender],"oracle: only proposer");
        _;
    }

    modifier  quorumSeted {
        require(quorum != 0,"oracle: quorum not set");
        _;
    }
    constructor(address _owner) {
        _transferOwnership(_owner);
    }

    function setQuorum(uint256 _quorum)external onlyOwner {
        require(_quorum != 0,"oracle: value_0");
        require(_quorum <= proposerCount,"oracle: quorum gt proposerCount");
        quorum = _quorum;
        emit SetQuorum(_quorum);
    }

    function setLightNode(address _lightNode)external onlyOwner {
        require(_lightNode != address(0),"oracle: address_0");
        lightNode = _lightNode;
        emit SetLightNode(_lightNode);
    }

    function updateProposer(address _proposer,bool _flag) external onlyOwner {
        require(_proposer != address(0),"oracle: address_0");
        require(proposers[_proposer] != _flag,"oracle: aready is");
        proposers[_proposer] = _flag;
        if(_flag) proposerCount ++;
        else proposerCount --;
        emit UpdateProposer(_proposer,_flag);
    }
    
    function propose(uint256 blockNum,bytes32 receiptRoot) external onlyProposer quorumSeted {
        address proposer = msg.sender;
        require(blockNum != 0,"oracle: value_0");
        require(receiptRoot != bytes32(''),"oracle: empty receipt root");
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum); 
        require(r == bytes32(''),"already update");
        require(records[proposer][blockNum] == bytes32(''),"oracle: proposer already propose this blockNum");
        records[proposer][blockNum] = receiptRoot;
        proposes[receiptRoot][blockNum] ++;
        if(proposes[receiptRoot][blockNum] >= quorum){
           _excute(blockNum,receiptRoot);
        }
        emit ProposeEvent(proposer,blockNum,receiptRoot);
    }

    function excute(uint256 blockNum,bytes32 receiptRoot) external {
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum); 
        require(r == bytes32(''),"already update");
        require(proposes[receiptRoot][blockNum] >= quorum,"oracle: approve not enough");
        _excute(blockNum,receiptRoot);
    }

    function _excute(uint256 _blockNum,bytes32 _receiptRoot) private {
        bytes memory u = abi.encode(_blockNum,_receiptRoot);
        IOracleLightNode(lightNode).updateBlockHeader(u);
        emit Excute(_blockNum,_receiptRoot);
    }

    function recoverPropose(address proposer,uint256 blockNum) external onlyOwner {
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum); 
        require(r == bytes32(''),"already update");
        require(records[proposer][blockNum] != bytes32(""),"oracle: proposer not propose this blockNum");
        delete records[proposer][blockNum];
        bytes32 receiptRoot = records[proposer][blockNum];
        proposes[receiptRoot][blockNum] --;
        emit RecoverPropose(proposer,blockNum);
    }

}