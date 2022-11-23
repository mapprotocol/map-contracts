// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/IWToken.sol";
import "./interface/IMAPToken.sol";
import "./interface/ILightClientManager.sol";
import "./interface/IMOSV2.sol";
import "./utils/TransferHelper.sol";
import "./utils/EvmDecoder.sol";
import "./utils/Utils.sol";


contract MAPOmnichainServiceRelayV2 is ReentrancyGuard, Initializable, IMOSV2 {
    using SafeMath for uint256;

    uint256 public immutable selfChainId = block.chainid;
    uint256 public nonce;
    address public wToken;        // native wrapped token

    ILightClientManager public lightClientManager;

    mapping(bytes32 => bool) public orderList;
    mapping(uint256 => bytes) public mosContracts;

    mapping(uint256 => mapping(bytes => bytes)) public sourceCorrespond;
    //MAP chain to target
    mapping(uint256 => mapping(bytes => bytes)) public mapCorrespond;

    event mapDepositIn(address indexed token, bytes from, address indexed to,
        bytes32 orderId, uint256 amount, uint256 fromChain);

    event mapTransferExecute(address indexed from, uint256 indexed fromChain, uint256 indexed toChain);

    function initialize(address _wToken, address _managerAddress) public initializer
    checkAddress(_wToken) checkAddress(_managerAddress) {
        wToken = _wToken;
        lightClientManager = ILightClientManager(_managerAddress);
    }


    receive() external payable {
        require(msg.sender == wToken, "only wToken");
    }


    modifier checkOrder(bytes32 orderId) {
        require(!orderList[orderId], "order exist");
        orderList[orderId] = true;
        _;
    }

    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }


    function setLightClientManager(address _managerAddress) external checkAddress(_managerAddress) {
        lightClientManager = ILightClientManager(_managerAddress);
    }

    function registerChain(uint256 _chainId, bytes memory _address) external {
        mosContracts[_chainId] = _address;
    }

    function emergencyWithdraw(address _token, address payable _receiver, uint256 _amount) external {
        _withdraw(_token, _receiver, _amount);
    }

    function transferOutToken(address _token, bytes memory _to, uint256 _amount, uint256 _toChain) external override {
        require(_toChain != selfChainId, "only other chain");
        require(IERC20(_token).balanceOf(msg.sender) >= _amount, "balance too low");

        TransferHelper.safeTransferFrom(_token, msg.sender, address(this), _amount);
        _transferOut(_token, msg.sender, _to, _amount, _toChain);
    }

    function transferOutNative(bytes memory _to, uint256 _toChain) external override payable {
        require(_toChain != selfChainId, "only other chain");
        uint256 amount = msg.value;
        require(amount > 0, "value too low");
        IWToken(wToken).deposit{value : amount}();
        _transferOut(wToken, msg.sender, _to, amount, _toChain);
    }

    function transferIn(uint256 _chainId, bytes memory _receiptProof) external nonReentrant {
        (bool success,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(_chainId, _receiptProof);
        require(success, message);
            IEvent.txLog[] memory logs = EvmDecoder.decodeTxLogs(logArray);
            for (uint256 i = 0; i < logs.length; i++) {
                IEvent.txLog memory log = logs[i];
                bytes32 topic = abi.decode(log.topics[0], (bytes32));
                if (topic == EvmDecoder.MAP_TRANSFEROUT_TOPIC) {
                    (bytes memory mosContract, IEvent.transferOutEvent memory outEvent) = EvmDecoder.decodeTransferOutLog(log);
                    //require(Utils.checkBytes(mosContract, mosContracts[_chainId]), "invalid mos contract");
                    if(Utils.checkBytes(mosContract, mosContracts[_chainId])) {
                        _transferIn(_chainId, outEvent);
                    }
                }
            }
        
        emit mapTransferExecute(msg.sender, _chainId, selfChainId);
    }

    function regToken(uint256 sourceChain, bytes memory sourceMapToken, bytes memory mapToken)
    external
    {
        sourceCorrespond[sourceChain][sourceMapToken] = mapToken;
        mapCorrespond[sourceChain][mapToken] = sourceMapToken;
    }

    function getTargetToken(uint256 sourceChain, bytes memory sourceToken, uint256 targetChain)
    public
    view
    returns (bytes memory mapToken){
        if(targetChain == selfChainId ){
            mapToken = sourceCorrespond[sourceChain][sourceToken];
        }else if(sourceChain == selfChainId){
            mapToken = mapCorrespond[targetChain][sourceToken];
        }else{
            mapToken = mapCorrespond[targetChain][sourceCorrespond[sourceChain][sourceToken]];
        }
    }


    function _getOrderId(address _token, address _from, bytes memory _to, uint256 _amount, uint256 _toChain) internal returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, _from, _to, _token, _amount, selfChainId, _toChain));
    }

    function _transferOut(address _token, address _from, bytes memory _to, uint256 _amount, uint256 _toChain) internal {

        bytes memory toToken = getTargetToken(selfChainId,Utils.toBytes(_token),_toChain);

        bytes32 orderId = _getOrderId(_token, _from, _to, _amount, _toChain);
        emit mapTransferOut(Utils.toBytes(_token), Utils.toBytes(_from), orderId, selfChainId, _toChain, _to, _amount, toToken);
    }

    function _transferIn(uint256 _chainId, IEvent.transferOutEvent memory _outEvent)
    internal checkOrder(_outEvent.orderId) {

        bytes memory tokenB = getTargetToken(_outEvent.fromChain, _outEvent.token,_outEvent.toChain);
    
        address token =  Utils.fromBytes(tokenB);

        uint256 mapOutAmount = _outEvent.amount;

        if (_outEvent.toChain == selfChainId) {
            address payable toAddress = payable(Utils.fromBytes(_outEvent.to));
            if (token == wToken) {
                TransferHelper.safeWithdraw(wToken, mapOutAmount);
                TransferHelper.safeTransferETH(toAddress, mapOutAmount);
            } else {
                require(IERC20(token).balanceOf(address(this)) >= mapOutAmount, "balance too low");
                TransferHelper.safeTransfer(token, toAddress, mapOutAmount);
            }
            emit mapTransferIn(token, _outEvent.from, _outEvent.orderId, _outEvent.fromChain, _outEvent.toChain,
                toAddress, mapOutAmount);
        }else {

            emit mapTransferOut(_outEvent.token, _outEvent.from, _outEvent.orderId, _outEvent.fromChain, _outEvent.toChain,
                _outEvent.to, mapOutAmount, tokenB);
        }
    }

    function _withdraw(address _token, address payable _receiver, uint256 _amount) internal {
        if (_token == wToken) {
            TransferHelper.safeWithdraw(wToken, _amount);
            TransferHelper.safeTransferETH(_receiver, _amount);
        } else {
            TransferHelper.safeTransfer(_token, _receiver, _amount);
        }
    }

}
