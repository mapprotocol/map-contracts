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

contract MAPCrossChainServiceRelay is ReentrancyGuard, Initializable, Pausable, IMCSRelay,UUPSUpgradeable {
    using SafeMath for uint;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint256 public nonce;

    ITokenRegister public tokenRegister;
    ILightClientManager public lightClientManager;
    IFeeCenter public feeCenter;

    address public wToken;        // native wrapped token

    uint public immutable selfChainId = block.chainid;

    mapping(bytes32 => bool) public orderList;

    mapping(address => uint) public transferFeeList;

    mapping(uint256 => mapping(address => uint)) public vaultBalance;

    mapping(bytes => uint256) mcsContracts;

    mapping(string => uint256) chainTable;

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

    struct nearDepositOutEvent {
        bytes token;
        bytes from;
        bytes order_id;
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
    checkAddress(_wToken) checkAddress(_managerAddress){
        wToken = _wToken;
        lightClientManager = ILightClientManager(_managerAddress);
        mapTransferOutTopic = keccak256(bytes('mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)'));
        mapDepositOutTopic = keccak256(bytes('mapDepositOut(address,bytes,bytes32,address,uint256)'));
        nearTransferOut = 0x4e87426fdd31a6df84975ed344b2c3fbd45109085f1557dff1156b300f135df8;
        nearDepositOut = 0x3ad224e3e42a516df08d1fca74990eac30205afb5287a46132a6975ce0b2cede;
        chainTable["near"] = 1313161555;
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
        require(_address != Address(0), "address is zero");
        _;
    }

    function setVaultBalance(uint256 tochain, address token, uint256 amount) external onlyOwner checkAddress(token) {
        vaultBalance[tochain][token] = amount;
    }

    function setTokenRegister(address _register) external onlyOwner checkAddress(_register){
        tokenRegister = ITokenRegister(_register);
    }

    function setLightClientManager(address _managerAddress) external onlyOwner checkAddress(_managerAddress){
        lightClientManager = ILightClientManager(_managerAddress);
    }

    function setMcsContract(uint256 _chainId, bytes memory _address) external onlyOwner checkAddress(_address){
        mcsContracts[_address] = _chainId;
    }

    function setPause() external onlyOwner {
        _pause();
    }

    function setUnpause() external onlyOwner {
        _unpause();
    }

    function getOrderID(address token, address from, bytes memory to, uint256 amount, uint256 toChainID) internal returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, from, to, token, amount, selfChainId, toChainID));
    }

    function setFeeCenter(address fee) external onlyOwner {
        feeCenter = IFeeCenter(fee);
    }

    function getFeeValue(uint256 amount, uint256 rate) pure public returns (uint){
        require(rate <= 1000000, 'Invalid rate value');
        return amount.mul(rate).div(1000000);
    }

    function checkAuthToken(address token) internal view returns (bool) {
        return tokenRegister.checkAuthToken(token);
    }

    function getToChainAmount(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount)
    internal view returns (uint256){
        return tokenRegister.getToChainAmount(token,fromChain,toChain,amount);
    }

    function getToChainAmountOther(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount)
    internal view returns (uint256){
        bytes memory tokenMap = getMapToken(token, fromChain);
        require(tokenMap.length > 0,"Token no register");
        return getToChainAmount(tokenMap, fromChain, toChain, amount);
    }

    function getMapToken(bytes memory fromToken, uint256 fromChain)
    internal view returns (bytes memory){
        return tokenRegister.getTargetToken(fromChain, fromToken, selfChainId);
    }

    function collectChainFee(uint256 amount, address token) private {
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

    function setChain(string memory name,uint256 chain) public onlyOwner {
        require(chain > 0,"id error");
        chainTable[name] = chain;
    }

    function transferIn(uint256 chainId, bytes memory receiptProof) external override {
        (bool sucess,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(chainId, receiptProof);
        require(sucess, message);
        if (chainId == chainTable["near"]) {
            (bytes memory mcsContract,nearTransferOutEvent memory _outEvent) = decodeNearLog(logArray);
            require(mcsContracts[mcsContract] > 0, "Illegal across the chain");
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
                    require(mcsContracts[mcsAddress] > 0, "Illegal across the chain");
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
        if(toChainId == chainTable["near"]){
            require(to.length >=2 || to.length <= 64,"near address error");
        }else {
            require(to.length == 20,"address error");
        }
        TransferHelper.safeTransferFrom(token, msg.sender, address(this), amount);
        uint256 fee = getChainFee(toChainId, token, amount);
        uint256 outAmount = amount.sub(fee,"sub error");
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
        if(toChainId == chainTable["near"]){
            require(to.length >=2 || to.length <= 64,"near address error");
        }else {
            require(to.length == 20,"address error");
        }
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

        if (_fromChain == chainTable["near"]) {
            (bytes memory mcsContract,nearDepositOutEvent memory _outEvent) = decodeNearDepositLog(logArray);
            require(mcsContracts[mcsContract] > 0, "Illegal across the chain");
            uint256 fromChain = _fromChain;
            bytes memory toChainToken = tokenRegister.getTargetToken(fromChain, _outEvent.token,selfChainId);
            uint256 outAmount = getToChainAmountOther(_outEvent.token,fromChain, selfChainId, _outEvent.amount);
            address payable toAddress = payable(_bytesToAddress(_outEvent.to));
            _depositIn(_bytesToAddress(toChainToken), _outEvent.from, toAddress, outAmount,bytes32(_outEvent.order_id), fromChain);
        } else {
            txLog[] memory logs = decodeTxLog(logArray);

            for (uint256 i = 0; i < logs.length; i++) {
                if (abi.decode(logs[i].topics[0], (bytes32)) == mapDepositOutTopic) {
                    require(mcsContracts[_addressToBytes(logs[i].addr)] > 0, "Illegal across the chain");
                    (address fromToken, bytes memory from,bytes32 orderId,address to,uint256 amount)
                    = abi.decode(logs[i].data, (address, bytes,bytes32, address,  uint256));
                    uint256 fromChain = _fromChain;
                    bytes memory _fromBytes = _addressToBytes(fromToken);
                    uint256 outAmount = getToChainAmountOther(_fromBytes, fromChain, selfChainId, amount);
                    _fromBytes = tokenRegister.getTargetToken(fromChain, _fromBytes, selfChainId);
                    address token = _bytesToAddress(_fromBytes);
                    _depositIn(token, from, payable(to), outAmount, orderId, fromChain);
                }
            }
        }
    }

    function _depositIn(address token, bytes memory from, address payable to, uint256 amount, bytes32 orderId, uint256 fromChain)
    internal checkOrder(orderId) {
        if (token == address(0)) {
            IWToken(wToken).deposit{value : amount}();
            token = wToken;
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


    function withdraw(address token, address payable receiver, uint256 amount) public onlyOwner {
        if (token == address(0)) {
            TransferHelper.safeWithdraw(wToken, amount);
            TransferHelper.safeTransferETH(receiver, amount);
        } else {
            TransferHelper.safeTransfer(token, receiver, amount);
        }
    }

    function _bytesToAddress(bytes memory bys) internal pure returns (address addr){
        assembly {
            addr := mload(add(bys, 20))
        }
    }

    function _addressToBytes(address self) internal pure returns (bytes memory b) {
        b = abi.encodePacked(self);
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

    function decodeNearDepositLog(bytes memory logsHash)
    public
    view
    returns (bytes memory executorId, nearDepositOutEvent memory _outEvent){
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

        _outEvent = nearDepositOutEvent({
        token : logList[0].toBytes(),
        from : logList[1].toBytes(),
        order_id : logList[2].toBytes(),
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
        require(extra.length >= 64,"Invalid extra result type");
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
    checkAuthToken(_admin){
        _changeAdmin(_admin);
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }



}