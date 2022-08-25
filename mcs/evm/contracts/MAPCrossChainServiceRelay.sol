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
import "./interface/ITokenRegister.sol";
import "./interface/ILightClientManager.sol";


contract MAPCrossChainServiceRelay is ReentrancyGuard, Role, Initializable, Pausable, IMCSRelay {
    using SafeMath for uint;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint256 public nonce;

    IERC20 public mapToken;
    //    ILightNode public lightNode;
    ITokenRegister public tokenRegister;
    ILightClientManager public lightClientManager;
    IFeeCenter public feeCenter;

    address public wToken;        // native wrapped token

    uint256 public selfChainId;

    // mapping(bytes32 => address) public tokenRegister;
    //Gas transfer fee charged by the target chain
    mapping(uint256 => uint) public chainGasFee;
    mapping(bytes32 => bool) public orderList;

    uint256 public chainGasFees;

    uint256 public transferFee;    // tranfer fee for every token, one in a million
    mapping(address => uint) public transferFeeList;

    mapping(address => bool) public authToken;

    mapping(uint256 => mapping(address => uint)) public vaultBalance;

    mapping(bytes => uint256) bridgeAddress;

    mapping(uint256 => uint) ChainIdTable;

    mapping(bytes => mapping(uint256 => uint256)) tokenOtherChainDecimals;

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    struct nearTransferOutEvent {
        bytes token;
        bytes from;
        bytes order_id;
        uint256 from_chain;
        uint256 to_chain;
        bytes to;
        uint256 amount;
        bytes to_chain_token;
    }

    event mapTransferOut(bytes token, bytes from, bytes32 orderId,
        uint256 fromChain, uint256 toChain, bytes to, uint256 amount, bytes toChainToken);

    event mapTransferIn(address indexed token, bytes indexed from, bytes32 indexed orderId,
        uint256 fromChain, uint256 toChain, address to, uint256 amount);

    event mapTokenRegister(bytes32 tokenID, address token);
    event mapDepositIn(address token, bytes from, address indexed to,
        bytes32 orderId, uint256 amount, uint256 fromChain);

    bytes32 public mapTransferOutTopic;
    bytes32 public nearTransferOut;
    bytes32 public mapDepositOutTopic;
    bytes32 public nearDepositOut;

    function initialize(address _wToken, address _mapToken, address _managerAddress) public initializer {
        uint256 _chainId;
        assembly {_chainId := chainid()}
        selfChainId = _chainId;
        wToken = _wToken;
        mapToken = IERC20(_mapToken);
        lightClientManager = ILightClientManager(_managerAddress);
        mapTransferOutTopic = keccak256(bytes('mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)'));
        mapDepositOutTopic = keccak256(bytes('mapDepositOut(bytes,address,bytes,bytes32,uint256)'));
        nearTransferOut = 0x4e87426fdd31a6df84975ed344b2c3fbd45109085f1557dff1156b300f135df8;
        nearDepositOut = '';
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

    function setVaultBalance(uint256 tochain, address token, uint256 amount) external onlyManager {
        vaultBalance[tochain][token] = amount;
    }

    function setTokenRegister(address _register) external onlyManager {
        tokenRegister = ITokenRegister(_register);
    }

    function setLightClientManager(address _managerAddress) external onlyManager {
        lightClientManager = ILightClientManager(_managerAddress);
    }

    function setBridageAddress(uint256 _chainId, bytes memory _addr) external onlyManager {
        bridgeAddress[_addr] = _chainId;
    }

    function setIdTable(uint256 _chainId, uint256 _id) external onlyManager {
        ChainIdTable[_id] = _chainId;
    }

    function setNearHash(bytes32 _hash) external onlyManager {
        nearTransferOut = _hash;
    }

    function setPause() external onlyManager {
        _pause();
    }

    function setUnpause() external onlyManager {
        _unpause();
    }

    function setTokenOtherChainDecimals(bytes memory selfToken, uint256 chainId, uint256 decimals) external onlyManager {
        tokenOtherChainDecimals[selfToken][chainId] = decimals;
    }

    function getOrderID(address token, address from, bytes memory to, uint256 amount, uint256 toChainID) public returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, from, to, token, amount, selfChainId, toChainID));
    }

    function setFeeCenter(address fee) external onlyManager {
        feeCenter = IFeeCenter(fee);
    }

    function addAuthToken(address[] memory token) external onlyManager {
        for (uint256 i = 0; i < token.length; i++) {
            authToken[token[i]] = true;
        }
    }

    function removeAuthToken(address[] memory token) external onlyManager {
        for (uint256 i = 0; i < token.length; i++) {
            authToken[token[i]] = false;
        }
    }

    function checkAuthToken(address token) internal view returns (bool) {
        return authToken[token];
    }

    function getFeeValue(uint256 amount, uint256 rate) pure public returns (uint){
        return amount.mul(rate).div(1000000);
    }

    function getToChainAmount(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount)
    internal view returns (uint256){
        uint256 decimalsFrom = tokenOtherChainDecimals[token][fromChain];
        uint256 decimalsTo = tokenOtherChainDecimals[token][toChain];
        return amount.mul(10 ** decimalsTo).div(10 ** decimalsFrom);
    }

    function getToChainAmountOther(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount)
    internal view returns (uint256){
        bytes memory tokenMap = getMapToken(token, fromChain);
        return getToChainAmount(tokenMap, fromChain, toChain, amount);
    }

    function getMapToken(bytes memory fromToken, uint256 fromChain)
    internal view returns (bytes memory){
        return tokenRegister.getTargetToken(fromChain, fromToken, selfChainId);
    }

    function collectChainFee(uint256 amount, address token) public {
        address transferToken = token;
        if (token == address(0)) {
            transferToken = wToken;
        }
        uint256 remaining = amount;
        if (amount > 0) {
            (address feeToken,uint256 rate) = feeCenter.getDistribute(0, token);
            uint256 out = getFeeValue(amount, rate);
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

    function setVaultValue(uint256 amount, uint256 fromChain, uint256 toChain, address token) internal {
        if (fromChain != selfChainId) {
            vaultBalance[fromChain][token] += amount;
        }
        if (toChain != selfChainId) {
            vaultBalance[toChain][token] -= amount;
        }
    }


    function getChainFee(uint256 toChainId, address token, uint256 amount) public view returns (uint256 out){
        if (token == address(0)) {
            token = wToken;
        }
        return feeCenter.getTokenFee(toChainId, token, amount);
    }


    function transferIn(uint256 chainId, bytes memory receiptProof) external override {
        (bool sucess,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(chainId, receiptProof);
        require(sucess, message);
        //near

        if (chainId == ChainIdTable[1]) {
            (bytes memory mcsContract,nearTransferOutEvent memory _outEvent) = decodeNearLog(logArray);
            require(bridgeAddress[mcsContract] > 0, "Illegal across the chain");
            bytes memory toChainToken = tokenRegister.getTargetToken(_outEvent.from_chain, _outEvent.token, _outEvent.to_chain);
            uint256 outAmount = getToChainAmountOther(_outEvent.token, _outEvent.from_chain, _outEvent.to_chain, _outEvent.amount);
            if (_outEvent.to_chain == selfChainId) {
                address payable toAddress = payable(_bytesToAddress(_outEvent.to));
                _transferIn(_bytesToAddress(toChainToken), _outEvent.from, toAddress, outAmount,
                    bytes32(_outEvent.order_id), _outEvent.from_chain, _outEvent.to_chain);
            } else {
                _transferInOtherChain(_outEvent.token, _outEvent.from, _outEvent.to, outAmount,
                    bytes32(_outEvent.order_id), _outEvent.from_chain, _outEvent.to_chain, toChainToken);
            }
        } else {

            txLog[] memory logs = decodeTxLog(logArray);

            for (uint256 i = 0; i < logs.length; i++) {
                txLog memory log = logs[i];
                bytes32 topic = abi.decode(log.topics[0], (bytes32));
                bytes memory mcsAddress = _addressToBytes(log.addr);
                if (topic == mapTransferOutTopic) {
                    require(bridgeAddress[mcsAddress] > 0, "Illegal across the chain");
                    //                address token = abi.decode(log.topics[1], (address));
                    // address from = abi.decode(log.topics[2], (address));
                    // bytes32 orderId = abi.decode(log.topics[3], (bytes32));
                    (bytes memory fromToken,bytes memory from,bytes32 orderId,uint256 fromChain,
                    uint256 toChain, bytes memory to, uint256 amount,)
                    = abi.decode(log.data, (bytes, bytes, bytes32, uint, uint, bytes, uint, bytes));
                    bytes memory toChainToken = tokenRegister.getTargetToken(fromChain, fromToken, toChain);
                    uint256 outAmount = getToChainAmountOther(fromToken, fromChain, toChain, amount);
                    if (toChain == selfChainId) {
                        address payable toAddress = payable(_bytesToAddress(to));
                        _transferIn(_bytesToAddress(toChainToken), from, toAddress, outAmount, orderId, fromChain, toChain);
                    } else {
                        _transferInOtherChain(fromToken, from, to, outAmount, orderId, fromChain, toChain, toChainToken);
                    }
                }
            }
        }
    }

    function transferOut(address toContract, uint256 toChain, bytes memory data) external override {

    }

    function transferOutToken(address token, bytes memory to, uint256 amount, uint256 toChainId) external override whenNotPaused {
        require(IERC20(token).balanceOf(msg.sender) >= amount, "balance too low");
        TransferHelper.safeTransferFrom(token, msg.sender, address(this), amount);
        uint256 fee = getChainFee(toChainId, token, amount);
        uint256 outAmount = amount.sub(fee);
        if (checkAuthToken(token)) {
            IMAPToken(token).burn(outAmount);
        }
        collectChainFee(fee, token);
        transferFeeList[token] = transferFeeList[token].add(amount).sub(outAmount);
        bytes32 orderId = getOrderID(token, msg.sender, to, outAmount, toChainId);
        setVaultValue(amount, selfChainId, toChainId, token);
        bytes memory toTokenAddress = tokenRegister.getTargetToken(selfChainId, _addressToBytes(token), toChainId);
        outAmount = getToChainAmount(_addressToBytes(token), selfChainId, toChainId, outAmount);
        emit mapTransferOut(_addressToBytes(token), _addressToBytes(msg.sender), orderId, selfChainId, toChainId, to, outAmount, toTokenAddress);
    }

    function transferOutNative(bytes memory to, uint256 toChainId) external override payable whenNotPaused {
        uint256 amount = msg.value;
        require(amount > 0, "value too low");
        IWToken(wToken).deposit{value : amount}();
        uint256 fee = getChainFee(toChainId, address(0), amount);
        uint256 outAmount = amount.sub(fee);
        collectChainFee(fee, address(0));
        transferFeeList[address(0)] = transferFeeList[address(0)].add(amount).sub(outAmount);
        bytes32 orderId = getOrderID(address(0), msg.sender, to, outAmount, toChainId);
        setVaultValue(amount, selfChainId, toChainId, address(0));
        bytes memory token = tokenRegister.getTargetToken(selfChainId, _addressToBytes(address(0)), toChainId);
        outAmount = getToChainAmount(_addressToBytes(address(0)), selfChainId, toChainId, outAmount);
        emit mapTransferOut(_addressToBytes(address(0)), _addressToBytes(msg.sender), orderId, selfChainId, toChainId, to, outAmount, token);
    }

    function _transferIn(address token, bytes memory from, address payable to, uint256 amount, bytes32 orderId, uint256 fromChain, uint256 toChain)
    internal checkOrder(orderId) nonReentrant whenNotPaused {
        uint256 fee = getChainFee(toChain, token, amount);
        uint256 outAmount = amount.sub(fee);
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
            emit mapTransferIn(token, from, orderId, fromChain, toChain, to, outAmount);
        }
        setVaultValue(amount, fromChain, toChain, token);
    }

    function _transferInOtherChain(bytes memory sourceToken, bytes memory from, bytes memory to, uint256 amount,
        bytes32 orderId, uint256 fromChain, uint256 toChain, bytes memory toChainToken)
    internal checkOrder(orderId) nonReentrant whenNotPaused {
        address token = _bytesToAddress(toChainToken);
        uint256 fee = getChainFee(toChain, token, amount);
        uint256 outAmount = amount.sub(fee);
        if (checkAuthToken(token)) {
            IMAPToken(token).mint(address(this), amount);
            IMAPToken(token).burn(outAmount);
        }
        emit mapTransferOut(sourceToken, from, orderId, fromChain, toChain, to, outAmount, toChainToken);
        address _token = _bytesToAddress(getMapToken(sourceToken, fromChain));
        setVaultValue(amount, fromChain, toChain, _token);
    }


    function depositIn(uint256 _fromChain, bytes memory receiptProof) external payable override nonReentrant whenNotPaused {
        (bool sucess,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(_fromChain, receiptProof);
        require(sucess, message);

        if (_fromChain == ChainIdTable[1]) {
            //todo will near
        } else {
            txLog[] memory logs = decodeTxLog(logArray);

            for (uint256 i = 0; i < logs.length; i++) {
                if (abi.decode(logs[i].topics[0], (bytes32)) == mapDepositOutTopic) {
                    require(bridgeAddress[_addressToBytes(logs[i].addr)] > 0, "Illegal across the chain");
                    (address fromToken, bytes memory from,address to,bytes32 orderId,uint256 amount)
                    = abi.decode(logs[i].data, (address, bytes, address, bytes32, uint256));
                    uint256 fromChain = _fromChain;
                    bytes memory _fromBytes = _addressToBytes(fromToken);
                    _fromBytes = tokenRegister.getTargetToken(fromChain, _fromBytes, selfChainId);
                    address token = _bytesToAddress(_fromBytes);
                    _depositIn(token, from, payable(to), amount, orderId, fromChain);
                }
            }
        }
    }

    function _depositIn(address token, bytes memory from, address payable to, uint256 amount, bytes32 orderId, uint256 fromChain)
    internal checkOrder(orderId) {
        if (token == address(0)) {
            IWToken(wToken).deposit{value : amount}();
            token == wToken;
        }
        address vaultTokenAddress = feeCenter.getVaultToken(token);
        require(vaultTokenAddress != address(0), "only vault token");
        if (checkAuthToken(token)) {
            IMAPToken(token).mint(vaultTokenAddress, amount);
        } else {
            TransferHelper.safeTransfer(token, vaultTokenAddress, amount);
        }
        IVault(vaultTokenAddress).stakingTo(amount, to);
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

    function _bytesToAddress(bytes memory bys) public pure returns (address addr){
        assembly {
            addr := mload(add(bys, 20))
        }
    }

    function _addressToBytes(address self) public pure returns (bytes memory b) {
        b = abi.encodePacked(self);
    }

    function decodeTxLog(bytes memory logsHash)
    public
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

    function decodeNearLog(bytes memory logsHash)
    public
    view
    returns (bytes memory executorId, nearTransferOutEvent memory _outEvent){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();

        executorId = ls[0].toBytes();

        bytes[] memory logs = new bytes[](ls[1].toList().length);
        for (uint256 i = 0; i < ls[1].toList().length; i++) {

            logs[i] = ls[1].toList()[i].toBytes();

        }
        bytes memory log;
        for (uint256 i = 0; i < logs.length; i++) {

            (bytes memory temp) = splitExtra(logs[i]);
            if (keccak256(temp) == nearTransferOut) {
                log = hexStrToBytes(logs[i]);
            }
        }

        RLPReader.RLPItem[] memory logList = log.toRlpItem().toList();

        _outEvent = nearTransferOutEvent({
        token : logList[0].toBytes(),
        from : logList[1].toBytes(),
        order_id : logList[2].toBytes(),
        from_chain : logList[3].toUint(),
        to_chain : logList[4].toUint(),
        to : logList[5].toBytes(),
        amount : logList[6].toUint(),
        to_chain_token : logList[7].toBytes()
        });

    }


    function hexStrToBytes(bytes memory _hexStr)
    public
    pure
    returns (bytes memory)
    {
        //Check hex string is valid
        if (
            _hexStr.length % 2 != 0 ||
            _hexStr.length < 4
        ) {
            revert("hexStrToBytes: invalid input");
        }

        bytes memory bytes_array = new bytes(_hexStr.length / 2 - 32);

        for (uint256 i = 64; i < _hexStr.length; i += 2) {
            uint8 tetrad1 = 16;
            uint8 tetrad2 = 16;

            //left digit
            if (
                uint8(_hexStr[i]) >= 48 && uint8(_hexStr[i]) <= 57
            ) tetrad1 = uint8(_hexStr[i]) - 48;

            //right digit
            if (
                uint8(_hexStr[i + 1]) >= 48 &&
                uint8(_hexStr[i + 1]) <= 57
            ) tetrad2 = uint8(_hexStr[i + 1]) - 48;

            //left A->F
            if (
                uint8(_hexStr[i]) >= 65 && uint8(_hexStr[i]) <= 70
            ) tetrad1 = uint8(_hexStr[i]) - 65 + 10;

            //right A->F
            if (
                uint8(_hexStr[i + 1]) >= 65 &&
                uint8(_hexStr[i + 1]) <= 70
            ) tetrad2 = uint8(_hexStr[i + 1]) - 65 + 10;

            //left a->f
            if (
                uint8(_hexStr[i]) >= 97 &&
                uint8(_hexStr[i]) <= 102
            ) tetrad1 = uint8(_hexStr[i]) - 97 + 10;

            //right a->f
            if (
                uint8(_hexStr[i + 1]) >= 97 &&
                uint8(_hexStr[i + 1]) <= 102
            ) tetrad2 = uint8(_hexStr[i + 1]) - 97 + 10;

            //Check all symbols are allowed
            if (tetrad1 == 16 || tetrad2 == 16)
                revert("hexStrToBytes: invalid input");

            bytes_array[i / 2 - 32] = bytes1(16 * tetrad1 + tetrad2);


        }

        return bytes_array;
    }


    function splitExtra(bytes memory extra)
    public
    pure
    returns (bytes memory newExtra){
        newExtra = new bytes(64);
        uint256 n = 0;
        for (uint256 i = 0; i < extra.length; i++) {
            if (i < 64) {
                newExtra[n] = extra[i];
            }
            n = n + 1;
        }
    }


}