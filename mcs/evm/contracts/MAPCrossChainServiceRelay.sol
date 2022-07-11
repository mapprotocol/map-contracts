// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/IWToken.sol";
import "./interface/IMAPToken.sol";
import "./interface/IFeeCenter.sol";
import "./utils/Role.sol";
import "./interface/IFeeCenter.sol";
import "./interface/IVault.sol";
import "./utils/TransferHelper.sol";
import "./interface/IMCSRelay.sol";
import "./utils/RLPReader.sol";
import "./interface/ILightNode.sol";


contract MAPCrossChainServiceRelay is ReentrancyGuard, Role, Initializable, Pausable, IMCSRelay {
    using SafeMath for uint;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint public nonce;

    IERC20 public mapToken;
    ILightNode public lightNode;
    address public wToken;          // native wrapped token

    uint public selfChainId;

    mapping(bytes32 => address) public tokenRegister;
    //Gas transfer fee charged by the target chain
    mapping(uint => uint) public chainGasFee;
    mapping(bytes32 => bool) orderList;

    uint public chainGasFees;


    uint public transferFee;    // tranfer fee for every token, one in a million
    mapping(address => uint) public transferFeeList;

    mapping(address => bool) public authToken;

    mapping(uint => mapping(address => uint)) public vaultBalance;

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    IFeeCenter public feeCenter;

    event mapTransferOut(address indexed token, address indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
    event mapTransferIn(address indexed token, bytes indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, address to, uint amount);

    event mapTokenRegister(bytes32 tokenID, address token);
    event mapDepositIn(address indexed token, address indexed from, address indexed to,
        bytes32 orderId, uint amount, uint fromChain);

    bytes32 mapTransferOutTopic = keccak256(bytes('mapTransferOut(address,address,bytes32,uint,uint,bytes,uint,bytes)'));
    //    bytes mapTransferInTopic = keccak256(bytes('mapTransferIn(address,address,bytes32,uint,uint,bytes,uint,bytes)'));

    function initialize(address _wToken, address _mapToken,address _lightNode) public initializer {
        uint _chainId;
        assembly {_chainId := chainid()}
        selfChainId = _chainId;
        wToken = _wToken;
        mapToken = IERC20(_mapToken);
        lightNode = ILightNode(_lightNode);
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(MANAGER_ROLE, msg.sender);
    }

    receive() external payable {
        require(msg.sender == wToken, "only wToken");
    }


    modifier checkOrder(bytes32 orderId) {
        require(!orderList[orderId], "order exist");
        orderList[orderId] = true;
        _;
    }

    function setVaultBalance(uint tochain, address token, uint amount) external onlyManager {
        vaultBalance[tochain][token] = amount;
    }

    function setPause() external onlyManager {
        _pause();
    }

    function setUnpause() external onlyManager {
        _unpause();
    }

    function getOrderID(address token, address from, bytes memory to, uint amount, uint toChainID) public returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, from, to, token, amount, selfChainId, toChainID));
    }

    function setFeeCenter(address fee) external onlyManager {
        feeCenter = IFeeCenter(fee);
    }

    function addAuthToken(address[] memory token) external onlyManager {
        for (uint i = 0; i < token.length; i++) {
            authToken[token[i]] = true;
        }
    }

    function removeAuthToken(address[] memory token) external onlyManager {
        for (uint i = 0; i < token.length; i++) {
            authToken[token[i]] = false;
        }
    }

    function checkAuthToken(address token) internal view returns (bool) {
        return authToken[token];
    }

    function getFeeValue(uint amount, uint rate) pure public returns (uint){
        return amount.mul(rate).div(1000000);
    }

    function collectChainFee(uint amount, address token) public {
        address transferToken = token;
        if (token == address(0)) {
            transferToken = wToken;
        }
        uint remaining = amount;
        if (amount > 0) {
            (address feeToken,uint rate) = feeCenter.getDistribute(0, token);
            uint out = getFeeValue(amount, rate);
            if (feeToken != address(0)) {
                TransferHelper.safeTransfer(transferToken, feeToken, out);
                remaining -= out;
            }
            (feeToken, rate) = feeCenter.getDistribute(1, token);
            out = getFeeValue(amount, rate);
            TransferHelper.safeTransfer(transferToken, feeToken, out);
            remaining -= out;
            if (remaining > 0) {
                TransferHelper.safeTransfer(transferToken, address(feeCenter), remaining);
            }
        }
    }

    function setVaultValue(uint amount, uint fromChain, uint toChain, address token) internal {
        if (fromChain != selfChainId) {
            vaultBalance[fromChain][token] += amount;
        }
        if (toChain != selfChainId) {
            vaultBalance[toChain][token] -= amount;
        }
    }


    function getChainFee(uint toChainId, address token, uint amount) public view returns (uint out){
        if (token == address(0)) {
            token = wToken;
        }
        return feeCenter.getTokenFee(toChainId, token, amount);
    }

    function transferIn(uint , bytes memory receiptProof) external override{
        (bool sucess,string memory message,bytes memory logArray) = lightNode.verifyProofData(receiptProof);
        require(sucess, message);
        txLog[] memory logs = decodeTxLog(logArray);

        for (uint i = 0; i < logs.length; i++) {
            txLog memory log = logs[i];
            bytes32 topic = abi.decode(log.topics[0], (bytes32));
            if (topic == mapTransferOutTopic) {
                //                address token = abi.decode(log.topics[1], (address));
                address from = abi.decode(log.topics[2], (address));
                bytes32 orderId = abi.decode(log.topics[3], (bytes32));
                (uint fromChain, uint toChain, bytes memory to, uint amount, bytes memory toChainToken)
                = abi.decode(log.data, (uint, uint, bytes, uint, bytes));
                address token = _bytesToAddress(toChainToken);
                address payable toAddress = payable(_bytesToAddress(to));
                _transferIn(token,from, toAddress, amount, orderId, fromChain, toChain);
            }
        }
    }

    function transferOut(address toContract, uint toChain, bytes memory data) external override{

    }

    function transferOutToken(address token, bytes memory to, uint amount, uint toChainId) external override whenNotPaused {
        require(IERC20(token).balanceOf(msg.sender) >= amount, "balance too low");
        TransferHelper.safeTransferFrom(token, msg.sender, address(this), amount);
        uint fee = getChainFee(toChainId, token, amount);
        uint outAmount = amount.sub(fee);
        if (checkAuthToken(token)) {
            IMAPToken(token).burn(outAmount);
        }
        collectChainFee(fee, token);
        transferFeeList[token] = transferFeeList[token].add(amount).sub(outAmount);
        bytes32 orderId = getOrderID(token, msg.sender, to, outAmount, toChainId);
        setVaultValue(amount, selfChainId, toChainId, token);
        emit mapTransferOut(token, msg.sender, orderId,selfChainId, toChainId, to, outAmount,_addressToBytes(address(0)));
    }

    function transferOutNative(bytes memory to, uint toChainId) external override payable whenNotPaused {
        uint amount = msg.value;
        require(amount > 0, "value too low");
        IWToken(wToken).deposit{value : amount}();
        uint fee = getChainFee(toChainId, address(0), amount);
        uint outAmount = amount.sub(fee);
        collectChainFee(fee, address(0));
        transferFeeList[address(0)] = transferFeeList[address(0)].add(amount).sub(outAmount);
        bytes32 orderId = getOrderID(address(0), msg.sender, to, outAmount, toChainId);
        setVaultValue(amount, selfChainId, toChainId, address(0));
        emit mapTransferOut(address(0),  msg.sender,orderId, selfChainId, toChainId, to, outAmount, _addressToBytes(address(0)));
    }

    function _transferIn(address token, address from, address payable to, uint amount, bytes32 orderId, uint fromChain, uint toChain)
    internal checkOrder(orderId) nonReentrant whenNotPaused {
        uint fee = getChainFee(toChain, token, amount);
        uint outAmount = amount.sub(fee);
        if (toChain == selfChainId) {
            if (token == address(0)) {
                TransferHelper.safeWithdraw(wToken, outAmount);
                TransferHelper.safeTransferETH(to, outAmount);
            } else if (checkAuthToken(token)) {
                IMAPToken(token).mint(address(this), amount);
                TransferHelper.safeTransfer(token, to, amount);
            } else {
                require(IERC20(token).balanceOf(address(this)) >= amount, "balance too low");
                TransferHelper.safeTransfer(token, to, outAmount);
            }
            collectChainFee(fee, token);
            emit mapTransferIn(address(0), _addressToBytes(from), orderId, fromChain, toChain, to, outAmount);
        } else {
            if (checkAuthToken(token)) {
                IMAPToken(token).burn(outAmount);
            }
            emit mapTransferOut(token, from, orderId, fromChain, toChain, _addressToBytes(to), outAmount, _addressToBytes(address(0)));
        }
        setVaultValue(amount, fromChain, toChain, token);
    }

    function depositIn(address token, address from, address payable to, uint amount, bytes32 orderId, uint fromChain)
    external payable override checkOrder(orderId) nonReentrant whenNotPaused {
        if (token == address(0)) {
            IWToken(wToken).deposit{value : amount}();
            token == wToken;
        }
        address vaultTokenAddress = feeCenter.getVaultToken(token);
        require(vaultTokenAddress != address(0), "only vault token");
        IVault vaultToken = IVault(vaultTokenAddress);
        if (checkAuthToken(token)) {
            IMAPToken(token).mint(vaultTokenAddress, amount);
        } else {
            TransferHelper.safeTransfer(token, vaultTokenAddress, amount);
        }
        vaultToken.stakingTo(amount, to);
        vaultBalance[fromChain][token] += amount;
        emit mapDepositIn(token, from, to, orderId, amount, fromChain);
    }


    function withdraw(address token, address payable receiver, uint256 amount) public onlyManager {
        if (token == address(0)) {
            IWToken(wToken).withdraw(amount);
            receiver.transfer(amount);
        } else {
            TransferHelper.safeTransfer(token, receiver, amount);
        }
    }

    function _bytesToAddress(bytes memory bys) internal pure returns (address addr){
        return abi.decode(bys,(address));
    }

    function _addressToBytes(address a) internal pure returns (bytes memory b) {
        return abi.encodePacked(a);
    }

    function decodeTxLog(bytes memory logsHash)
    internal
    pure
    returns (txLog[] memory _txLogs){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            bytes[] memory topic = new bytes[](ls[i].toList()[1].toList().length);
            for (uint256 j = 0; j < ls[i].toList()[1].toList().length; j++) {
                topic[j] = ls[i].toList()[1].toList()[j].toBytes();
            }
            _txLogs[i] = txLog({
            addr : ls[i].toList()[0].toAddress(),
            topics : topic,
            data : ls[i].toList()[2].toBytes()
            });
        }
    }

}