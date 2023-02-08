// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "./interface/IWToken.sol";
import "./interface/IMAPToken.sol";
import "./interface/IVaultTokenV2.sol";
import "./interface/ITokenRegisterV2.sol";
import "./interface/ILightClientManager.sol";
import "./interface/IMOSV3.sol";
import "./utils/TransferHelper.sol";
import "./utils/EvmDecoder.sol";
import "./utils/NearDecoder.sol";
import "./utils/Utils.sol";
//import "hardhat/console.sol";


contract MAPOmnichainServiceRelayV2 is ReentrancyGuard, Initializable, Pausable, IMOSV3, UUPSUpgradeable {
    using SafeMath for uint256;
    using Address for address;

    struct Rate {
        address receiver;
        uint rate;
    }
    enum chainType{
        NULL,
        EVM,
        NEAR
    }

    uint256 public immutable selfChainId = block.chainid;
    uint256 public nonce;
    address public wToken;        // native wrapped token
    //id : 0 VToken  1:relayer
    ITokenRegisterV2 public tokenRegister;
    ILightClientManager public lightClientManager;

    mapping(uint => Rate) public distributeRate;
    mapping(bytes32 => bool) public orderList;
    mapping(uint256 => bytes) public mosContracts;
    mapping(uint256 => chainType) public chainTypes;

    mapping(bytes => bool) public executeWhiteList;

    event mapTransferRelay(uint256 indexed fromChain, uint256 indexed toChain, bytes32 orderId,
        address token, bytes from, bytes to, uint256 amount);

    event mapDepositIn(uint256 indexed fromChain, uint256 indexed toChain, address indexed token, bytes32 orderId,
        bytes from, address to, uint256 amount);

    event mapTransferExecute(uint256 indexed fromChain, uint256 indexed toChain, address indexed from);

    event SetTokenRegister(address tokenRegister);
    event SetLightClientManager(address lightClient);
    event RegisterChain(uint256 _chainId, bytes _address, chainType _type);
    event SetDistributeRate(uint _id, address _to, uint _rate);
    event AddWhiteList(bytes _executeAddress, bool _enable);

    function initialize(address _wToken, address _managerAddress) public initializer
    checkAddress(_wToken) checkAddress(_managerAddress) {
        wToken = _wToken;
        lightClientManager = ILightClientManager(_managerAddress);
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
        require(msg.sender == _getAdmin(), "mosRelay :: only admin");
        _;
    }

    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }


    function setPause() external onlyOwner {
        _pause();
    }

    function setUnpause() external onlyOwner {
        _unpause();
    }

    function setTokenManager(address _register) external onlyOwner checkAddress(_register) {
        tokenRegister = ITokenRegisterV2(_register);
        emit SetTokenRegister(_register);
    }

    function setLightClientManager(address _managerAddress) external onlyOwner checkAddress(_managerAddress) {
        lightClientManager = ILightClientManager(_managerAddress);
        emit SetLightClientManager(_managerAddress);
    }

    function registerChain(uint256 _chainId, bytes memory _address, chainType _type) external onlyOwner {
        mosContracts[_chainId] = _address;
        chainTypes[_chainId] = _type;
        emit RegisterChain(_chainId, _address, _type);
    }

    function addWhiteList(bytes memory _executeAddress,bool _enable) external onlyOwner {

        executeWhiteList[_executeAddress] = _enable;
        emit AddWhiteList(_executeAddress,_enable);
    }

    // withdraw deposit token using vault token.
    function withdraw(address _vaultToken, uint256 _vaultAmount) external {
        require(_vaultToken != address(0), "vault token not registered");
        address token = IVaultTokenV2(_vaultToken).getTokenAddress();
        address vaultToken = tokenRegister.getVaultToken(token);
        require(_vaultToken == vaultToken, "Invalid vault token");

        uint256 amount = IVaultTokenV2(vaultToken).getTokenAmount(_vaultAmount);
        IVaultTokenV2(vaultToken).withdraw(selfChainId, _vaultAmount, msg.sender);
        _withdraw(token, payable(msg.sender), amount);
    }

    function emergencyWithdraw(address _token, address payable _receiver, uint256 _amount) external onlyOwner checkAddress(_receiver) {
        _withdraw(_token, _receiver, _amount);
    }

    function setDistributeRate(uint _id, address _to, uint _rate) external onlyOwner checkAddress(_to) {
        require(_id < 3, "Invalid rate id");

        distributeRate[_id] = Rate(_to, _rate);

        require((distributeRate[0].rate).add(distributeRate[1].rate).add(distributeRate[2].rate) <= 1000000, 'invalid rate value');
        emit SetDistributeRate(_id, _to, _rate);
    }

    function transferOut(uint256 _toChain,CallData memory _callData) external  override
    payable
    nonReentrant
    whenNotPaused
    returns(bool)
    {
        require(_toChain != selfChainId, "only other chain");
        require(_callData.gasLimit >= 21000 ,"Execution gas too low");
        require(_callData.gasLimit < 1000000000000000000 ,"Execution gas too high");
        uint amount = msg.value;
        if(amount > 0 && amount == _callData.value){
            IWToken(wToken).deposit{value : amount}();
        }

        bytes32 orderId = _getOrderId(msg.sender, _callData.target, _toChain);

        bytes memory callData = abi.encode(_callData);

        emit mapDataOut(selfChainId, _toChain, orderId, callData);
        return true;
    }

    function transferOutToken(address _token, bytes memory _to, uint256 _amount, uint256 _toChain) external override whenNotPaused {
        require(Utils.isValidAddress(_to, uint256(chainTypes[_toChain])), "to address is error");
        require(_toChain != selfChainId, "only other chain");
        require(IERC20(_token).balanceOf(msg.sender) >= _amount, "balance too low");
        require(_token.isContract(), "token is not contract");

        TransferHelper.safeTransferFrom(_token, msg.sender, address(this), _amount);
        _transferOut(_token, msg.sender, _to, _amount, _toChain);
    }

    function transferOutNative(bytes memory _to, uint256 _toChain) external override payable whenNotPaused {
        require(Utils.isValidAddress(_to, uint256(chainTypes[_toChain])), "to address is error");
        require(_toChain != selfChainId, "only other chain");
        uint256 amount = msg.value;
        require(amount > 0, "value too low");
        IWToken(wToken).deposit{value : amount}();
        _transferOut(wToken, msg.sender, _to, amount, _toChain);
    }

    function depositToken(address _token, address _to, uint _amount)
    external override
    nonReentrant
    whenNotPaused
    checkAddress(_token)
    {
        require(IERC20(_token).balanceOf(msg.sender) >= _amount, "balance too low");

        require(_token.isContract(), "token is not contract");

        TransferHelper.safeTransferFrom(_token, msg.sender, address(this), _amount);

        _deposit(_token, Utils.toBytes(msg.sender), _to, _amount, bytes32(""), selfChainId);
    }

    function depositNative(address _to) external override payable nonReentrant whenNotPaused {
        uint256 amount = msg.value;
        require(amount > 0, "value too low");
        IWToken(wToken).deposit{value : amount}();
        _deposit(wToken, Utils.toBytes(msg.sender), _to, amount, bytes32(""), selfChainId);
    }

    function transferIn(uint256 _chainId, bytes memory _receiptProof) external nonReentrant whenNotPaused {
        (bool success,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(_chainId, _receiptProof);
        require(success, message);
        if (chainTypes[_chainId] == chainType.NEAR) {
            (bytes memory mosContract, IEvent.transferOutEvent[] memory outEvents) = NearDecoder.decodeNearLog(logArray);
            for (uint i = 0; i < outEvents.length; i++) {
                IEvent.transferOutEvent memory outEvent = outEvents[i];
                if (outEvent.toChain == 0) {continue;}
                require(Utils.checkBytes(mosContract, mosContracts[_chainId]), "invalid mos contract");
                _transferIn(_chainId, outEvent);
            }
        } else if (chainTypes[_chainId] == chainType.EVM) {
            IEvent.txLog[] memory logs = EvmDecoder.decodeTxLogs(logArray);
            for (uint256 i = 0; i < logs.length; i++) {
                IEvent.txLog memory log = logs[i];
                bytes32 topic = abi.decode(log.topics[0], (bytes32));
                if (topic == EvmDecoder.MAP_TRANSFEROUT_TOPIC) {
                    (bytes memory mosContract, IEvent.transferOutEvent memory outEvent) = EvmDecoder.decodeTransferOutLog(log);
                    //require(Utils.checkBytes(mosContract, mosContracts[_chainId]), "invalid mos contract");
                    if (Utils.checkBytes(mosContract, mosContracts[_chainId])) {
                        _transferIn(_chainId, outEvent);
                    }
                }
                if (topic == EvmDecoder.MAP_DATA_TOPIC) {
                    (, IEvent.dataOutEvent memory outEvent) = EvmDecoder.decodeDataLog(log);
                    _executeIn(outEvent);
                }
            }
        } else {
            require(false, "chain type error");
        }
        emit mapTransferExecute(_chainId, selfChainId, msg.sender);
    }

    function depositIn(uint256 _chainId, bytes memory _receiptProof) external payable nonReentrant whenNotPaused {
        (bool success,string memory message,bytes memory logArray) = lightClientManager.verifyProofData(_chainId, _receiptProof);
        require(success, message);
        if (chainTypes[_chainId] == chainType.NEAR) {
            (bytes memory mosContract, IEvent.depositOutEvent[] memory depositEvents) = NearDecoder.decodeNearDepositLog(logArray);

            for (uint i = 0; i < depositEvents.length; i++) {
                IEvent.depositOutEvent memory depositEvent = depositEvents[i];
                if (depositEvent.toChain == 0) {continue;}
                require(Utils.checkBytes(mosContract, mosContracts[_chainId]), "invalid mos contract");
                _depositIn(_chainId, depositEvent);
            }
        } else if (chainTypes[_chainId] == chainType.EVM) {
            IEvent.txLog[] memory logs = EvmDecoder.decodeTxLogs(logArray);
            for (uint256 i = 0; i < logs.length; i++) {
                if (abi.decode(logs[i].topics[0], (bytes32)) == EvmDecoder.MAP_DEPOSITOUT_TOPIC) {
                    (bytes memory mosContract, IEvent.depositOutEvent memory depositEvent) = EvmDecoder.decodeDepositOutLog(logs[i]);
                    if (Utils.checkBytes(mosContract, mosContracts[_chainId])) {
                        _depositIn(_chainId, depositEvent);
                    }
                }
            }
        } else {
            require(false, "chain type error");
        }
        emit mapTransferExecute(_chainId, selfChainId, msg.sender);
    }

    function _executeIn(IEvent.dataOutEvent memory _outEvent) internal checkOrder(_outEvent.orderId)  {

        if(_outEvent.toChain == selfChainId){

            CallData memory cData = abi.decode(_outEvent.cData,(CallData));

            address callDataAddress = Utils.fromBytes(cData.target);

            bool success;

            if(executeWhiteList[cData.target]){
                (success, ) = callDataAddress.call{gas:cData.gasLimit}(cData.callData);
            }

            emit mapExecuteIn(_outEvent.fromChain, _outEvent.toChain,_outEvent.orderId, success);

        }else{

            emit mapDataOut(selfChainId,_outEvent.toChain,_outEvent.orderId,_outEvent.cData);
        }

    }


    function getFee(uint256 _id, uint256 _amount) view public returns (uint256, address){
        Rate memory rate = distributeRate[_id];
        return (_amount.mul(rate.rate).div(1000000), rate.receiver);
    }


    function _getOrderId(address _from, bytes memory _to, uint256 _toChain) internal returns (bytes32){
        return keccak256(abi.encodePacked(address(this), nonce++, selfChainId, _toChain, _from, _to));
    }

    function _collectFee(address _token, uint256 _mapAmount, uint256 _fromChain, uint256 _toChain) internal returns (uint256, uint256) {
        address token = _token;
        address vaultToken = tokenRegister.getVaultToken(token);
        require(vaultToken != address(0), "vault token not registered");

        uint256 fee = tokenRegister.getTokenFee(token, _mapAmount, _toChain);

        uint256 mapOutAmount = 0;
        uint256 outAmount = 0;
        if (_mapAmount > fee) {
            mapOutAmount = _mapAmount - fee;
            outAmount = tokenRegister.getToChainAmount(token, mapOutAmount, _toChain);
        } else {
            fee = _mapAmount;
        }

        uint256 otherFee = 0;
        if (fee > 0) {
            (uint256 vaultFee,) = getFee(0, fee);
            otherFee = fee - vaultFee;

            (uint256 out, address receiver) = getFee(1, fee);
            if (out > 0 && receiver != address(0)) {
                _withdraw(token, payable(receiver), out);
            }

            (uint256 protocolFee, address protocolReceiver) = getFee(2, fee);
            if (protocolFee > 0 && protocolReceiver != address(0)) {
                _withdraw(token, payable(protocolReceiver), protocolFee);
            }
        }

        IVaultTokenV2(vaultToken).transferToken(_fromChain, _mapAmount, _toChain, mapOutAmount, selfChainId, otherFee);

        return (mapOutAmount, outAmount);
    }

    function _transferOut(address _token, address _from, bytes memory _to, uint256 _amount, uint256 _toChain) internal {
        bytes memory toToken = tokenRegister.getToChainToken(_token, _toChain);
        require(!Utils.checkBytes(toToken, bytes("")), "out token not registered");

        bytes32 orderId = _getOrderId(_from, _to, _toChain);
        emit mapTransferRelay(selfChainId, _toChain, orderId, _token, Utils.toBytes(_from), _to, _amount);

        (uint256 mapOutAmount, uint256 outAmount) = _collectFee(_token, _amount, selfChainId, _toChain);

        if (tokenRegister.checkMintable(_token)) {
            IMAPToken(_token).burn(mapOutAmount);
        }

        emit mapTransferOut(selfChainId, _toChain, orderId, Utils.toBytes(_token), Utils.toBytes(_from), _to, outAmount, toToken);
    }

    function _transferIn(uint256 _chainId, IEvent.transferOutEvent memory _outEvent)
    internal checkOrder(_outEvent.orderId) {
        require(_chainId == _outEvent.fromChain, "invalid chain id");
        address token = tokenRegister.getRelayChainToken(_outEvent.fromChain, _outEvent.token);
        require(token != address(0), "map token not registered");
        bytes memory toChainToken;
        if (_outEvent.toChain == selfChainId) {
            toChainToken = Utils.toBytes(token);
        } else {
            toChainToken = tokenRegister.getToChainToken(token, _outEvent.toChain);
            require(!Utils.checkBytes(toChainToken, bytes("")), "out token not registered");
        }

        uint256 mapAmount = tokenRegister.getRelayChainAmount(token, _outEvent.fromChain, _outEvent.amount);
        if (tokenRegister.checkMintable(token)) {
            IMAPToken(token).mint(address(this), mapAmount);
        }

        emit mapTransferRelay(_outEvent.fromChain, _outEvent.toChain, _outEvent.orderId, token, _outEvent.from, _outEvent.to, mapAmount);

        (uint256 mapOutAmount, uint256 outAmount) = _collectFee(token, mapAmount, _outEvent.fromChain, _outEvent.toChain);

        if (_outEvent.toChain == selfChainId) {
            address payable toAddress = payable(Utils.fromBytes(_outEvent.to));
            if (token == wToken) {
                TransferHelper.safeWithdraw(wToken, mapOutAmount);
                TransferHelper.safeTransferETH(toAddress, mapOutAmount);
            } else {
                require(IERC20(token).balanceOf(address(this)) >= mapOutAmount, "balance too low");
                TransferHelper.safeTransfer(token, toAddress, mapOutAmount);
            }
            emit mapTransferIn(_outEvent.fromChain, _outEvent.toChain, _outEvent.orderId, token, _outEvent.from,
                toAddress, mapOutAmount);
        } else {
            if (tokenRegister.checkMintable(token)) {
                IMAPToken(token).burn(mapOutAmount);
            }

            emit mapTransferOut(_outEvent.fromChain, _outEvent.toChain, _outEvent.orderId, _outEvent.token, _outEvent.from,
                _outEvent.to, outAmount, toChainToken);
        }
    }

    function _depositIn(uint256 _chainId, IEvent.depositOutEvent memory _depositEvent)
    internal checkOrder(_depositEvent.orderId) {
        require(_chainId == _depositEvent.fromChain, "invalid chain id");
        require(selfChainId == _depositEvent.toChain, "invalid chain id");
        address token = tokenRegister.getRelayChainToken(_depositEvent.fromChain, _depositEvent.token);
        require(token != address(0), "map token not registered");

        uint256 mapAmount = tokenRegister.getRelayChainAmount(token, _depositEvent.fromChain, _depositEvent.amount);
        if (tokenRegister.checkMintable(token)) {
            IMAPToken(token).mint(address(this), mapAmount);
        }

        _deposit(token, _depositEvent.from, Utils.fromBytes(_depositEvent.to), mapAmount, _depositEvent.orderId, _depositEvent.fromChain);
    }

    function _deposit(address _token, bytes memory _from, address _to, uint256 _amount, bytes32 _orderId, uint256 _fromChain)
    internal {
        address vaultToken = tokenRegister.getVaultToken(_token);
        require(vaultToken != address(0), "vault token not registered");

        IVaultTokenV2(vaultToken).deposit(_fromChain, _amount, _to);
        emit mapDepositIn(_fromChain, selfChainId, _token, _orderId, _from, _to, _amount);
    }

    function _withdraw(address _token, address payable _receiver, uint256 _amount) internal {
        if (_token == wToken) {
            TransferHelper.safeWithdraw(wToken, _amount);
            TransferHelper.safeTransferETH(_receiver, _amount);
        } else {
            TransferHelper.safeTransfer(_token, _receiver, _amount);
        }
    }


    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "MAPOmnichainServiceRelay: only Admin can upgrade");
    }

    function changeAdmin(address _admin) external onlyOwner checkAddress(_admin) {
        _changeAdmin(_admin);
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
