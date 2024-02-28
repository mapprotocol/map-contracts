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
    address public proposer;
    uint256 public quorum;
    uint256 public approveCount;

    struct Propose {
        bytes32 receiptRoot;
        address[] aprovers;
    }

    uint256[] public pendingProposes;
    mapping(address => bool) public approvers;
    mapping(uint256 => Propose) public blockNumToPropose;

    event SetQuorum(uint256 indexed _quorum);
    event SetProposer(address indexed _proposer);
    event RemovePropose(uint256 indexed blockNum);
    event SetLightNode(address indexed _lightNode);
    event Approve(uint256 indexed blockNum,address indexed approver);
    event UpdateApprover(address indexed _approver,bool indexed _flag);
    event Excute(uint256 indexed _blockNum,bytes32 indexed _receiptRoot);
    event ProposeEvent(uint256 indexed blockNum,bytes32 indexed receiptRoot);
    event UpdatePropose(uint256 indexed BlockNum,bytes32 indexed receiptRoot);

    modifier  onlyProposer {
        require(msg.sender == proposer,"oracle: only proposer");
        _;
    }

    modifier  onlyApprover {
        require(approvers[msg.sender],"oracle: only approver");
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
        require(_quorum <= approveCount,"oracle: quorum gt approveCount");
        quorum = _quorum;
        emit SetQuorum(_quorum);
    }

    function setLightNode(address _lightNode)external onlyOwner {
        require(_lightNode != address(0),"oracle: address_0");
        lightNode = _lightNode;
        emit SetLightNode(_lightNode);
    }

    function setProposer(address _proposer) external onlyOwner {
        require(_proposer != address(0),"oracle: address_0");
        proposer = _proposer;
        emit SetProposer(_proposer);
    }

    function updateApprover(address _approver,bool _flag) external onlyOwner {
        require(_approver != address(0),"oracle: address_0");
        require(approvers[_approver] != _flag,"oracle: aready is");
        approvers[_approver] = _flag;
        if(_flag) approveCount ++;
        else approveCount --;
        emit UpdateApprover(_approver,_flag);
    }
    
    function propose(uint256 blockNum,bytes32 receiptRoot) external onlyProposer quorumSeted {
        require(blockNum != 0,"oracle: value_0");
        require(receiptRoot != bytes32(''),"oracle: empty receipt root");
        bytes32 r = IOracleLightNode(lightNode).receiptRoots(blockNum); 
        require(blockNumToPropose[blockNum].receiptRoot == bytes32(''),"oracle: aready add");
        require(r == bytes32(''),"already update");
        blockNumToPropose[blockNum].receiptRoot = receiptRoot;
        pendingProposes.push(blockNum);
        emit ProposeEvent(blockNum,receiptRoot);
    }

    function updatePropose(uint256 index,bytes32 receiptRoot) external onlyProposer{
        require(receiptRoot != bytes32(''),"oracle: empty receipt root");
        uint256 blockNum = pendingProposes[index];
        Propose storage p = blockNumToPropose[blockNum];
        p.receiptRoot = receiptRoot;
        delete p.aprovers;
        emit UpdatePropose(blockNum,receiptRoot);
    }

    function removePropose(uint256 index) external onlyProposer {
        uint256 blockNum = pendingProposes[index];
        delete blockNumToPropose[blockNum];
        _delPendingPropose(index);
        emit RemovePropose(blockNum);
    }

    function approve(uint256 index) external  onlyApprover{
        uint256 blockNum = pendingProposes[index];
        Propose storage p = blockNumToPropose[blockNum];
        uint256 len = p.aprovers.length;
        for (uint i = 0; i < len; i++) {
            require(msg.sender != p.aprovers[i],"already approve");
        } 
        p.aprovers.push(msg.sender);
        emit Approve(blockNum,msg.sender);
        if(p.aprovers.length >= quorum){
            _excute(blockNum,p.receiptRoot); 
            delete blockNumToPropose[blockNum];
            _delPendingPropose(index);
        }
    }

    function excute(uint256 index) external {
        uint256 blockNum = pendingProposes[index];
        Propose storage p = blockNumToPropose[blockNum];
        require(p.aprovers.length >= quorum,"oracle: approve not enough");
        _excute(blockNum,p.receiptRoot);
        delete blockNumToPropose[blockNum];
    }

    function _excute(uint256 _blockNum,bytes32 _receiptRoot) private {
        bytes memory u = abi.encode(_blockNum,_receiptRoot);
        IOracleLightNode(lightNode).updateBlockHeader(u);
        emit Excute(_blockNum,_receiptRoot);
    }

    function _delPendingPropose(uint256 index) private {
          uint256 lastIndex = pendingProposes.length - 1;
          if(index != lastIndex){
            pendingProposes[index] = pendingProposes[lastIndex];
          }
          pendingProposes.pop();        
    }
    
    struct  PendingPropose {
        uint256 blockNum;
        Propose propose;
    }
    function getPendingProposes() external view returns(PendingPropose[] memory p){
        uint256 len = pendingProposes.length;
        p = new PendingPropose[](len);
        for (uint i = 0; i < len; i++) {
           uint256 blockNum = pendingProposes[i];
           p[i] = PendingPropose({
              blockNum: blockNum,
              propose: blockNumToPropose[blockNum]
            });
        }
    }
    
}