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
    IERC20 public mapToken;
    ILightNode public lightNode;
    address public wToken;          // native wrapped token

    uint public immutable selfChainId = block.chainid;
    uint public nearChainId;

    mapping(bytes32 => address) public tokenRegister;
    //Gas transfer fee charged by the target chain
    mapping(uint => uint) public chainGasFee;
    mapping(bytes32 => bool) public orderList;

    uint public chainGasFees;
    mapping(address => bool) public authToken;

    mapping(address => uint256) public bridgeAddress;

    //Can storage tokens be cross-chain?
    mapping(address => mapping(uint => bool)) canBridgeToken;


    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    event mapTransferOut(bytes token, bytes from, bytes32 orderId,
        uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
    event mapTransferIn(address indexed token, bytes indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, address to, uint amount);

    event mapTransferOutData(bytes indexed toContract, address indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, bytes data);
    event mapTransferInData(bytes indexed toContract, address indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, bytes data);

    event mapTokenRegister(bytes32 tokenID, address token);
    event mapDepositOut(address token, bytes from, bytes32 orderId, address to, uint256 amount);


    //bytes32 public mapTransferOutTopic = keccak256(bytes('mapTransferOut(address,address,bytes32,uint,uint,bytes,uint,bytes)'));
    bytes32 public constant mapTransferOutTopic = keccak256(abi.encodePacked("mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)"));
    //    bytes mapTransferInTopic = keccak256(bytes('mapTransferIn(address,address,bytes32,uint,uint,bytes,uint,bytes)'));

    function initialize(address _wToken, address _mapToken, address _lightNode) public initializer {
        wToken = _wToken;
        nearChainId = 1313161555;
        mapToken = IERC20(_mapToken);
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

    function setBridge(address _bridge, uint256 _num) public onlyOwner {
        bridgeAddress[_bridge] = _num;
    }

    function checkAuthToken(address token) public view returns (bool) {
        return authToken[token];
    }

    function setCanBridgeToken(address token, uint chainId, bool canBridge) public onlyOwner {
        canBridgeToken[token][chainId] = canBridge;
    }

    function setChainId(uint256 _id) public onlyOwner {
        require(_id > 0,"id error");
        nearChainId = _id;
    }


    function transferIn(uint, bytes memory receiptProof) external override nonReentrant whenNotPaused {
        (bool sucess,string memory message,bytes memory logArray) = lightNode.verifyProofData(receiptProof);
        require(sucess, message);
        txLog[] memory logs = decodeTxLog(logArray);

        for (uint i = 0; i < logs.length; i++) {
            txLog memory log = logs[i];
            bytes32 topic = abi.decode(log.topics[0], (bytes32));
            if (topic == mapTransferOutTopic) {
                require(bridgeAddress[log.addr] > 0, "Illegal across the chain");
                //                address token = abi.decode(log.topics[1], (address));
                // address from = abi.decode(log.topics[2], (address));
                // bytes32 orderId = abi.decode(log.topics[3], (bytes32));
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
        if(toChain == nearChainId){
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
        if(toChain == nearChainId){
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

    function getInfo(bytes memory hash) public view returns(
        address to,
        uint256 value,
        bytes memory data,
    //        uint256 operation,
    //        uint256 safeTxGas,
    //        uint256 baseGas,
    //        uint256 gasPrice,
        address gasToken,
        address  refundReceiver,
        bytes memory signatures){
        //to,value,data
        (to,value,data,,,,,gasToken,refundReceiver,signatures) = abi.decode(hash,(
            address,uint256,bytes,uint256,uint256,uint256,uint256,address,address,bytes
            ));
    }

}