// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
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
import "./interface/IVault.sol";
import "./utils/TransferHelper.sol";
import "./interface/IMCSRelay.sol";
import "./utils/RLPReader.sol";
import "./interface/ITokenRegister.sol";
import "./interface/ILightClientManager.sol";
import "./utils/AddressUtils.sol";


contract MAPCrossChainServiceRelay is ReentrancyGuard, Initializable, Pausable, IMCSRelay, UUPSUpgradeable {
    using SafeMath for uint256;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint256 public nonce;

    ITokenRegister public tokenRegister;
    ILightClientManager public lightClientManager;
    IFeeCenter public feeCenter;

    address public wToken;        // native wrapped token

    mapping(address => bool) public authToken;

    uint256 public immutable selfChainId = block.chainid;

    mapping(bytes32 => bool) public orderList;

    mapping(uint256 => mapping(address => uint256)) public vaultBalance;

    mapping(uint256 => bytes) mcsContracts;

    enum chainType{
        NULL,
        EVM,
        NEAR
    }
    mapping(uint256 => chainType) public chainTypes;

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    struct transferOutEvent {
        bytes token;
        bytes from;
        bytes32 orderId;
        uint256 fromChain;
        uint256 toChain;
        bytes to;
        uint256 amount;
        bytes toChainToken;
    }

    struct depositOutEvent {
        bytes token;
        bytes from;
        bytes32 orderId;
        bytes to;
        uint256 amount;
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

    function initialize(address _wToken, address _managerAddress) public initializer
    checkAddress(_wToken) checkAddress(_managerAddress) {
        wToken = _wToken;
        lightClientManager = ILightClientManager(_managerAddress);
        mapTransferOutTopic = keccak256(bytes('mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)'));
        mapDepositOutTopic = keccak256(bytes('mapDepositOut(address,bytes,bytes32,address,uint256)'));
        nearTransferOut = 0x4e87426fdd31a6df84975ed344b2c3fbd45109085f1557dff1156b300f135df8;
        nearDepositOut = 0x3ad224e3e42a516df08d1fca74990eac30205afb5287a46132a6975ce0b2cede;
        _changeAdmin(msg.sender);
    }

    receive() external payable {
        require(msg.sender == wToken, "only wToken");
    }


    modifier checkOrder(bytes32 orderId) {
        require(!orderList[orderId], "order exist");
        orderList[orderId] = true;
        _;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "mcsRelay :: only admin");
        _;
    }

    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }

    function setVaultBalance(uint256 tochain, address token, uint256 amount) external onlyOwner {
        vaultBalance[tochain][token] = amount;
    }

    function setTokenRegister(address _register) external onlyOwner checkAddress(_register) {
        tokenRegister = ITokenRegister(_register);
    }

    function setLightClientManager(address _managerAddress) external onlyOwner checkAddress(_managerAddress) {
        lightClientManager = ILightClientManager(_managerAddress);
    }

    function setMcsContract(uint256 _chainId, bytes memory _address, chainType _type) external onlyOwner {
        mcsContracts[_chainId] = _address;
        chainTypes[_chainId] = _type;
    }

    function setPause() external onlyOwner {
        _pause();
    }

    function setUnpause() external onlyOwner {
        _unpause();
    }

    function addAuthToken(address[] memory token)
    external
    onlyOwner {
        for (uint256 i = 0; i < token.length; i++) {
            require(token[i] != address(0), "token is zero");
            authToken[token[i]] = true;
        }
    }

    function removeAuthToken(address[] memory token)
    external
    onlyOwner {
        for (uint256 i = 0; i < token.length; i++) {
            authToken[token[i]] = false;
        }
    }

    function checkAuthToken(address token)
    internal
    view returns (bool) {
        return authToken[token];
    }

    function getOrderID(address token, address from, bytes memory to, uint256 amount, uint256 toChainID) internal returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, from, to, token, amount, selfChainId, toChainID));
    }

    function setFeeCenter(address fee) external onlyOwner {
        feeCenter = IFeeCenter(fee);
    }

    function getFeeValue(uint256 amount, uint256 rate) pure public returns (uint256){
        require(rate <= 1000000, 'Invalid rate value');
        return amount.mul(rate).div(1000000);
    }

    function getToChainAmount(address token, uint256 fromChain, uint256 toChain, uint256 amount)
    internal view returns (uint256){
        return tokenRegister.getToChainAmount(token, fromChain, toChain, amount);
    }

    function getToChainAmountOther(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount)
    internal view returns (uint256){
        address tokenMap = getMapToken(token, fromChain);
        require(tokenMap != address(0), "Token no register");
        return getToChainAmount(tokenMap, fromChain, toChain, amount);
    }

    function getMapToken(bytes memory fromToken, uint256 fromChain)
    internal view returns (address){
        return AddressUtils.fromBytes(tokenRegister.getTargetToken(fromChain, fromToken, selfChainId));
    }

    function collectChainFee(uint256 amount, address token) private {
        address transferToken = token;
        uint256 remaining = amount;
        if (amount > 0) {
            (address feeToken,uint256 rate) = feeCenter.getDistribute(0, token);
            uint256 out = getFeeValue(amount, rate);
            if (feeToken != address(0)) {
                IVault(feeToken).addFee(out);
                remaining -= out;
            }
            (feeToken, rate) = feeCenter.getDistribute(1, token);
            out = getFeeValue(amount, rate);
            TransferHelper.safeTransfer(transferToken, feeToken, out);
            remaining -= out;
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
        return feeCenter.getTokenFee(toChainId, token, amount);
    }

    function transferIn(uint256 chainId, bytes memory receiptProof) external override {
        (bool success,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(chainId, receiptProof);
        require(success, message);
        if (chainTypes[chainId] == chainType.NEAR) {
            (bytes memory mcsContract,transferOutEvent memory _outEvent) = decodeNearLog(logArray);
            require(checkBytes(mcsContract, mcsContracts[chainId]), "Illegal across the chain");
            bytes memory toChainToken = tokenRegister.getTargetToken(_outEvent.fromChain, _outEvent.token, _outEvent.toChain);
            uint256 outAmount = getToChainAmountOther(_outEvent.token, _outEvent.fromChain, _outEvent.toChain, _outEvent.amount);
            if (_outEvent.toChain == selfChainId) {
                address payable toAddress = payable(AddressUtils.fromBytes(_outEvent.to));
                _transferIn(AddressUtils.fromBytes(toChainToken), _outEvent.from, toAddress, outAmount,
                    bytes32(_outEvent.orderId), _outEvent.fromChain, _outEvent.toChain);
            } else {
                _transferInOtherChain(_outEvent, outAmount, toChainToken);
            }
        } else if (chainTypes[chainId] == chainType.EVM) {
            txLog[] memory logs = decodeTxLog(logArray);
            for (uint256 i = 0; i < logs.length; i++) {
                txLog memory log = logs[i];
                bytes32 topic = abi.decode(log.topics[0], (bytes32));
                bytes memory mcsContract = AddressUtils.toBytes(log.addr);
                if (topic == mapTransferOutTopic) {
                    require(checkBytes(mcsContract, mcsContracts[chainId]), "Illegal across the chain");
                    transferOutEvent memory _outEvent;
                    (_outEvent.token, _outEvent.from, _outEvent.orderId, _outEvent.fromChain,
                    _outEvent.toChain, _outEvent.to, _outEvent.amount,)
                    = abi.decode(log.data, (bytes, bytes, bytes32, uint256, uint256, bytes, uint256, bytes));
                    bytes memory toChainToken = tokenRegister.getTargetToken(_outEvent.fromChain, _outEvent.token, _outEvent.toChain);
                    uint256 outAmount = getToChainAmountOther(_outEvent.token, _outEvent.fromChain, _outEvent.toChain, _outEvent.amount);
                    if (_outEvent.toChain == selfChainId) {
                        address payable toAddress = payable(AddressUtils.fromBytes(_outEvent.to));
                        _transferIn(AddressUtils.fromBytes(toChainToken), _outEvent.from, toAddress, outAmount, _outEvent.orderId,
                            _outEvent.fromChain, _outEvent.toChain);
                    } else {
                        _transferInOtherChain(_outEvent, outAmount, toChainToken);
                    }
                }
            }
        } else {
            require(true, "chain type error");
        }
    }

    function transferOut(address toContract, uint256 toChain, bytes memory data) external override {

    }

    function transferOutToken(address token, bytes memory to, uint256 amount, uint256 toChainId) external override whenNotPaused {
        _transferOut(token, to, amount, toChainId);
    }

    function transferOutNative(bytes memory to, uint256 toChainId) external override payable whenNotPaused {
        uint256 amount = msg.value;
        _transferOut(wToken, to, amount, toChainId);
    }

    function _transferOut(address token, bytes memory to, uint256 amount, uint256 toChainId) internal {
        bytes memory toToken = tokenRegister.getTargetToken(selfChainId, AddressUtils.toBytes(token), toChainId);
        require(!checkBytes(toToken, bytes("")), "token not register");
        uint256 fee = getChainFee(toChainId, token, amount);
        uint256 outAmount = amount.sub(fee, "sub error");
        if (token == wToken) {
            require(amount > 0, "value too low");
            IWToken(wToken).deposit{value : amount}();
        } else {
            require(IERC20(token).balanceOf(msg.sender) >= amount, "balance too low");
            TransferHelper.safeTransferFrom(token, msg.sender, address(this), amount);
            if (checkAuthToken(token)) {
                IMAPToken(token).burn(outAmount);
            }
        }
        collectChainFee(fee, token);
        bytes32 orderId = getOrderID(token, msg.sender, to, outAmount, toChainId);
        setVaultValue(amount, selfChainId, toChainId, token);
        outAmount = getToChainAmount(token, selfChainId, toChainId, outAmount);
        emit mapTransferOut(AddressUtils.toBytes(token), AddressUtils.toBytes(msg.sender), orderId, selfChainId, toChainId, to, outAmount, toToken);
    }

    function _transferIn(address token, bytes memory from, address payable to, uint256 amount, bytes32 orderId, uint256 fromChain, uint256 toChain)
    internal checkOrder(orderId) nonReentrant whenNotPaused {
        uint256 fee = getChainFee(toChain, token, amount);
        uint256 outAmount = amount.sub(fee);
        if (toChain == selfChainId) {
            if (token == wToken) {
                TransferHelper.safeWithdraw(wToken, outAmount);
                TransferHelper.safeTransferETH(to, outAmount);
            } else if (checkAuthToken(token)) {
                IMAPToken(token).mint(address(this), outAmount);
                TransferHelper.safeTransfer(token, to, outAmount);
            } else {
                require(IERC20(token).balanceOf(address(this)) >= outAmount, "balance too low");
                TransferHelper.safeTransfer(token, to, outAmount);
            }
            collectChainFee(fee, token);
            emit mapTransferIn(token, from, orderId, fromChain, toChain, to, outAmount);
        }
        setVaultValue(amount, fromChain, toChain, token);
    }

    function _transferInOtherChain(transferOutEvent memory outEvent, uint256 outAmount, bytes memory toChainToken)
    internal checkOrder(outEvent.orderId) nonReentrant whenNotPaused {
        address token = AddressUtils.fromBytes(toChainToken);

        address mapToken = getMapToken(outEvent.token, outEvent.fromChain);
        uint256 fee = getChainFee(outEvent.toChain, token, outAmount);
        uint256 outMap = getToChainAmount(mapToken, outEvent.fromChain, selfChainId, outAmount);
        uint256 feeMap = getToChainAmount(mapToken, outEvent.fromChain, selfChainId, fee);

        outAmount = outAmount.sub(fee);
        if (checkAuthToken(mapToken)) {
            IMAPToken(mapToken).mint(address(this), outMap);
            IMAPToken(mapToken).burn(outMap.sub(feeMap));
        }
        collectChainFee(feeMap, token);
        setVaultValue(outMap, outEvent.fromChain, outEvent.toChain, mapToken);
        emit mapTransferOut(outEvent.token, outEvent.from, outEvent.orderId, outEvent.fromChain, outEvent.toChain,
            outEvent.to, outAmount, toChainToken);
    }


    function checkBytes(bytes memory b1, bytes memory b2) internal view returns (bool){
        return keccak256(b1) == keccak256(b2);
    }

    function depositIn(uint256 _fromChain, bytes memory receiptProof) external payable override nonReentrant whenNotPaused {
        (bool success,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(_fromChain, receiptProof);
        require(success, message);

        if (chainTypes[_fromChain] == chainType.NEAR) {
            (bytes memory mcsContract,depositOutEvent memory _outEvent) = decodeNearDepositLog(logArray);
            require(checkBytes(mcsContract, mcsContracts[_fromChain]), "Illegal across the chain");
            uint256 fromChain = _fromChain;
            bytes memory toChainToken = tokenRegister.getTargetToken(fromChain, _outEvent.token, selfChainId);
            uint256 outAmount = getToChainAmountOther(_outEvent.token, fromChain, selfChainId, _outEvent.amount);
            address payable toAddress = payable(AddressUtils.fromBytes(_outEvent.to));
            _depositIn(AddressUtils.fromBytes(toChainToken), _outEvent.from, toAddress, outAmount, bytes32(_outEvent.orderId), fromChain);
        } else if (chainTypes[_fromChain] == chainType.EVM) {
            txLog[] memory logs = decodeTxLog(logArray);
            for (uint256 i = 0; i < logs.length; i++) {
                if (abi.decode(logs[i].topics[0], (bytes32)) == mapDepositOutTopic) {
                    require(logs[i].addr == AddressUtils.fromBytes(mcsContracts[_fromChain]), "Illegal across the chain");
                    (address fromToken, bytes memory from,bytes32 orderId,address to,uint256 amount)
                    = abi.decode(logs[i].data, (address, bytes, bytes32, address, uint256));
                    uint256 fromChain = _fromChain;
                    bytes memory _fromBytes = AddressUtils.toBytes(fromToken);
                    uint256 outAmount = getToChainAmountOther(_fromBytes, fromChain, selfChainId, amount);
                    _fromBytes = tokenRegister.getTargetToken(fromChain, _fromBytes, selfChainId);
                    address token = AddressUtils.fromBytes(_fromBytes);
                    _depositIn(token, from, payable(to), outAmount, orderId, fromChain);
                }
            }
        } else {
            require(true, "chain type error");
        }
    }


    function deposit(address token, uint256 amount) external {
        if (token == wToken) {
            IWToken(wToken).deposit{value : amount}();
        }
        address vaultTokenAddress = feeCenter.getVaultToken(token);
        require(vaultTokenAddress != wToken, "only vault token");
        TransferHelper.safeTransfer(token, address(this), amount);
        IVault(vaultTokenAddress).stakingTo(amount, msg.sender);
        vaultBalance[selfChainId][token] += amount;
        emit mapDepositIn(token,AddressUtils.toBytes(msg.sender), msg.sender, bytes32(""), amount, selfChainId);
    }

    function _depositIn(address token, bytes memory from, address payable to, uint256 amount, bytes32 orderId, uint256 fromChain)
    internal checkOrder(orderId) {
        if (token == wToken) {
            IWToken(wToken).deposit{value : amount}();
        }
        address vaultTokenAddress = feeCenter.getVaultToken(token);
        require(vaultTokenAddress != wToken, "only vault token");
        if (checkAuthToken(token)) {
            IMAPToken(token).mint(address(this), amount);
        }
        //todo token from ?
        //        else {
        //            TransferHelper.safeTransfer(token, address(this), amount);
        //        }
        IVault(vaultTokenAddress).stakingTo(amount, to);
        vaultBalance[fromChain][token] += amount;
        emit mapDepositIn(token, from, to, orderId, amount, fromChain);
    }


    function withdraw(address token, address payable receiver, uint256 amount) public onlyOwner {
        if (token == wToken) {
            TransferHelper.safeWithdraw(wToken, amount);
            TransferHelper.safeTransferETH(receiver, amount);
        } else {
            TransferHelper.safeTransfer(token, receiver, amount);
        }
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

    function decodeNearLog(bytes memory logsHash)
    internal
    view
    returns (bytes memory executorId, transferOutEvent memory _outEvent){
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

        _outEvent = transferOutEvent({
        token : logList[0].toBytes(),
        from : logList[1].toBytes(),
        orderId : bytes32(logList[2].toBytes()),
        fromChain : logList[3].toUint(),
        toChain : logList[4].toUint(),
        to : logList[5].toBytes(),
        amount : logList[6].toUint(),
        toChainToken : logList[7].toBytes()
        });

    }

    function decodeNearDepositLog(bytes memory logsHash)
    public
    view
    returns (bytes memory executorId, depositOutEvent memory _outEvent){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();

        executorId = ls[0].toBytes();

        bytes[] memory logs = new bytes[](ls[1].toList().length);
        for (uint256 i = 0; i < ls[1].toList().length; i++) {

            logs[i] = ls[1].toList()[i].toBytes();

        }
        bytes memory log;
        for (uint256 i = 0; i < logs.length; i++) {

            (bytes memory temp) = splitExtra(logs[i]);
            if (keccak256(temp) == nearDepositOut) {
                log = hexStrToBytes(logs[i]);
            }
        }

        RLPReader.RLPItem[] memory logList = log.toRlpItem().toList();

        _outEvent = depositOutEvent({
        token : logList[0].toBytes(),
        from : logList[1].toBytes(),
        orderId : bytes32(logList[2].toBytes()),
        to : logList[3].toBytes(),
        amount : logList[4].toUint()
        });

    }


    function hexStrToBytes(bytes memory _hexStr)
    internal
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
    internal
    pure
    returns (bytes memory newExtra){
        require(extra.length >= 64, "Invalid extra result type");
        newExtra = new bytes(64);
        for (uint256 i = 0; i < 64; i++) {
            newExtra[i] = extra[i];
        }
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address)
    internal
    view
    override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }

    function changeAdmin(address _admin)
    public onlyOwner
    checkAddress(_admin) {
        _changeAdmin(_admin);
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }


}