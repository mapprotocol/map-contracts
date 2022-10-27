// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./interface/IWToken.sol";
import "./interface/IMAPToken.sol";
import "./interface/IFeeCenter.sol";
import "./utils/TransferHelper.sol";
import "./interface/IMCS.sol";
import "./interface/ILightNode.sol";
import "./utils/RLPReader.sol";

contract MapCrossChainService is ReentrancyGuard, Initializable, Pausable, IMCS,UUPSUpgradeable {
    using SafeMath for uint;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint public nonce;
    ILightNode public lightNode;
    address public wToken;          // native wrapped token

    uint public immutable selfChainId = block.chainid;

    mapping(bytes32 => bool) public orderList;
    mapping(address => bool) public authToken;

    address public mscRelay;

    //Can storage tokens be cross-chain?
    mapping(address => mapping(uint => bool)) canBridgeToken;

    mapping(string => uint256) chainTable;

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    event mapTransferOut(bytes token, bytes from, bytes32 orderId,
        uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
    event mapTransferIn(address indexed token, bytes indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, address to, uint amount);

    event mapDepositOut(address token, bytes from, bytes32 orderId, address to, uint256 amount);

    bytes32 public constant mapTransferOutTopic
    = keccak256(abi.encodePacked("mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)"));

    modifier checkAddress(address _address){
        require(_address != address (0), "address is zero");
        _;
    }

    function initialize(address _wToken, address _lightNode)
    public initializer checkAddress(_wToken) checkAddress(_lightNode){
        wToken = _wToken;
        chainTable["near"] = 1313161555;
        lightNode = ILightNode(_lightNode);
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

    modifier checkCanBridge(address token, uint chainId) {
        require(canBridgeToken[token][chainId], "token not can bridge");
        _;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    function setPause() external onlyOwner {
        _pause();
    }

    function setUnpause() external onlyOwner {
        _unpause();
    }

    function getOrderID(address token, address from, bytes memory to, uint amount, uint toChainID) internal returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, from, to, token, amount, selfChainId, toChainID));
    }

    function addAuthToken(address[] memory token) external onlyOwner {
        for (uint i = 0; i < token.length; i++) {
            authToken[token[i]] = true;
        }
    }

    function removeAuthToken(address[] memory token) external onlyOwner {
        for (uint i = 0; i < token.length; i++) {
            authToken[token[i]] = false;
        }
    }

    function setMcsRelay(address _relay) public onlyOwner checkAddress(_relay) {
        mscRelay = _relay;
    }

    function checkAuthToken(address token) public view returns (bool) {
        return authToken[token];
    }

    function setCanBridgeToken(address token, uint chainId, bool canBridge) public onlyOwner {
        canBridgeToken[token][chainId] = canBridge;
    }

    function setChain(string memory name,uint256 chain) public onlyOwner {
        require(chain > 0,"id error");
        chainTable[name] = chain;
    }

    function transferIn(uint, bytes memory receiptProof) external override nonReentrant whenNotPaused {
        (bool sucess,string memory message,bytes memory logArray) = lightNode.verifyProofData(receiptProof);
        require(sucess, message);
        txLog[] memory logs = decodeTxLog(logArray);

        for (uint i = 0; i < logs.length; i++) {
            txLog memory log = logs[i];
            bytes32 topic = abi.decode(log.topics[0], (bytes32));
            if (topic == mapTransferOutTopic) {
                require(mscRelay == log.addr, "Illegal across the chain");
                (,bytes memory from,bytes32 orderId,uint fromChain, uint toChain, bytes memory to, uint amount, bytes memory toChainToken)
                = abi.decode(log.data, (bytes, bytes, bytes32, uint, uint, bytes, uint, bytes));
                address token = _bytesToAddress(toChainToken);
                address payable toAddress = payable(_bytesToAddress(to));
                _transferIn(token, from, toAddress, amount, orderId, fromChain, toChain);
            }
        }
    }


    function transferOut(address toContract, uint toChain, bytes memory data) external override whenNotPaused {

    }

    function transferOutToken(address token, bytes memory toAddress, uint amount, uint toChain)
    external override
    whenNotPaused
    checkCanBridge(token, toChain)
    {
        if(toChain == chainTable["near"]){
            require(toAddress.length >=2 || toAddress.length <= 64,"near address error");
        }else {
            require(toAddress.length == 20,"address error");
        }
        bytes32 orderId = getOrderID(token, msg.sender, toAddress, amount, toChain);
        require(IERC20(token).balanceOf(msg.sender) >= amount, "balance too low");
        if (checkAuthToken(token)) {
            IMAPToken(token).burnFrom(msg.sender, amount);
        } else {
            TransferHelper.safeTransferFrom(token, msg.sender, address(this), amount);
        }
        emit mapTransferOut(_addressToBytes(token), _addressToBytes(msg.sender), orderId, selfChainId, toChain, toAddress, amount, _addressToBytes(address(0)));
    }


    function transferOutNative(bytes memory toAddress, uint toChain)
    external override payable
    whenNotPaused
    checkCanBridge(address(0), toChain) {
        if(toChain == chainTable["near"]){
            require(toAddress.length >=2 || toAddress.length <= 64,"near address error");
        }else {
            require(toAddress.length == 20,"address error");
        }
        uint amount = msg.value;
        require(amount > 0, "balance is zero");
        bytes32 orderId = getOrderID(address(0), msg.sender, toAddress, amount, toChain);
        IWToken(wToken).deposit{value : amount}();
        emit mapTransferOut(_addressToBytes(address(0)), _addressToBytes(msg.sender), orderId, selfChainId, toChain, toAddress, amount, _addressToBytes(address(0)));
    }


    function depositOutToken(address token, address from, address to, uint amount) external override payable whenNotPaused {
        require(msg.sender == from, "from only sender");
        bytes32 orderId = getOrderID(token, from, _addressToBytes(to), amount, 22776);
        //        require(IERC20(token).balanceOf(from) >= amount, "balance too low");
        TransferHelper.safeTransferFrom(token, from, address(this), amount);
        emit mapDepositOut(token, _addressToBytes(from),orderId,to,amount);
    }

    function depositOutNative(address from, address to) external override payable whenNotPaused {
        require(msg.sender == from, "from only sender");
        uint amount = msg.value;
        bytes32 orderId = getOrderID(address(0), from, _addressToBytes(to), amount, 22776);
        require(msg.value >= amount, "balance too low");
        IWToken(wToken).deposit{value : amount}();
        emit mapDepositOut(address(0), _addressToBytes(from),orderId, to, amount);
    }

    function _transferIn(address token, bytes memory from, address payable to, uint amount, bytes32 orderId, uint fromChain, uint toChain)
    internal checkOrder(orderId) {
        if (token == address(0)) {
            TransferHelper.safeWithdraw(wToken, amount);
            TransferHelper.safeTransferETH(to, amount);
        } else if (checkAuthToken(token)) {
            IMAPToken(token).mint(to, amount);
        } else {
            TransferHelper.safeTransfer(token, to, amount);
        }
        emit mapTransferIn(token, from, orderId, fromChain, toChain, to, amount);
    }


    function withdraw(address token, address payable receiver, uint256 amount) public onlyOwner {
        if (token == address(0)) {
            TransferHelper.safeWithdraw(wToken, amount);
            TransferHelper.safeTransferETH(receiver, amount);
        } else {
            IERC20(token).transfer(receiver, amount);
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

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address)
    internal
    view
    override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }

    function changeAdmin(address _admin) public onlyOwner {
        require(_admin != address(0), "zero address");

        _changeAdmin(_admin);
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }

}