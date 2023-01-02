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
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "./interface/IWToken.sol";
import "./interface/IMAPToken.sol";
import "./interface/IFeeCenter.sol";
import "./utils/TransferHelper.sol";
import "./interface/IMCS.sol";
import "./interface/ILightNode.sol";
import "./utils/RLPReader.sol";

contract MapCrossChainService is ReentrancyGuard, Initializable, Pausable, IMCS, UUPSUpgradeable {
    using SafeMath for uint;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;
    using Address for address;

    uint public nonce;
    ILightNode public lightNode;
    address public wToken;          // native wrapped token

    uint public immutable selfChainId = block.chainid;

    mapping(bytes32 => bool) public orderList;

    mapping(address => bool) public authToken;

    //mapping(address => uint256) public bridgeAddress;
    address public relayContract;
    uint256 public relayChainId;

    //Can storage tokens be cross-chain?
    mapping(uint256 => mapping(address => bool)) canBridgeToken;

    address private _pendingAdmin;

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    event mapTransferOut(bytes token, bytes from, bytes32 orderId,
        uint fromChain, uint toChain, bytes to, uint amount, bytes toChainToken);
    event mapTransferIn(address indexed token, bytes indexed from, bytes32 indexed orderId,
        uint fromChain, uint toChain, address to, uint amount);


    event mapTokenRegister(bytes32 tokenID, address token);
    event mapDepositOut(address indexed token, bytes from, bytes32 orderId,
        uint256 fromChain, uint256 toChain, address to, uint256 amount);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event AdminTransferred(address indexed previous, address indexed newAdmin);

    event AddAuthToken(address[] token);
    event RemoveAuthToken(address[] token);
    event SetBridge(address _bridge,uint256 _num);
    event SetCanBridgeToken(address token,uint256 chainId,bool canBridge);


    bytes32 public constant mapTransferOutTopic = keccak256(abi.encodePacked("mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)"));


    function initialize(address _wToken, address _lightNode)
    public initializer checkAddress(_wToken) checkAddress(_lightNode) {
        wToken = _wToken;
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

    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }


    modifier checkCanBridge(address token, uint chainId) {
        require(canBridgeToken[chainId][token], "token not can bridge");
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
            require(token[i] != address(0), "address is zero");
            authToken[token[i]] = true;
        }
        emit AddAuthToken(token);
    }

    function removeAuthToken(address[] memory token) external onlyOwner {
        for (uint i = 0; i < token.length; i++) {
            authToken[token[i]] = false;
        }
        emit RemoveAuthToken(token);
    }

    function setBridge(address _bridge, uint256 _num) public onlyOwner checkAddress(_bridge) {
        relayChainId = _num;
        relayContract = _bridge;
        emit SetBridge(_bridge,_num);
    }

    function checkAuthToken(address token) public view returns (bool) {
        return authToken[token];
    }

    function setCanBridgeToken(address token, uint chainId, bool canBridge) public onlyOwner {
        canBridgeToken[chainId][token] = canBridge;
        emit SetCanBridgeToken(token,chainId,canBridge);
    }


    function transferIn(uint chainId, bytes memory receiptProof) external override nonReentrant whenNotPaused {
        (bool success,string memory message,bytes memory logArray) = lightNode.verifyProofData(receiptProof);
        require(success, message);
        txLog[] memory logs = decodeTxLog(logArray);

        require(chainId == relayChainId, "Illegal across the chain id");

        for (uint i = 0; i < logs.length; i++) {
            txLog memory log = logs[i];
            bytes32 topic = abi.decode(log.topics[0], (bytes32));
            if (topic == mapTransferOutTopic) {
                require(log.addr == relayContract, "Illegal across the chain");
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

    function transferOutToken(address token, bytes memory toAddress, uint amount, uint toChain)
    external override
    whenNotPaused
    checkCanBridge(token, toChain)
    checkAddress(token)
    {
        require(toChain != selfChainId, "only other chain");
        require(token.isContract(),"token is not contract");
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
        uint amount = msg.value;
        require(amount > 0, "balance is zero");
        bytes32 orderId = getOrderID(address(0), msg.sender, toAddress, amount, toChain);
        IWToken(wToken).deposit{value : amount}();
        emit mapTransferOut(_addressToBytes(address(0)), _addressToBytes(msg.sender), orderId, selfChainId, toChain, toAddress, amount, _addressToBytes(address(0)));
    }


    function depositOutToken(address token, address from, address to, uint amount)
    external override
    whenNotPaused
    checkCanBridge(token, relayChainId)
    checkAddress(token)
    {
        require(msg.sender == from, "from only sender");
        require(token.isContract(),"token is not contract");
        bytes32 orderId = getOrderID(token, from, _addressToBytes(to), amount, relayChainId);
        require(IERC20(token).balanceOf(from) >= amount, "balance too low");
        TransferHelper.safeTransferFrom(token, from, address(this), amount);
        emit mapDepositOut(token, _addressToBytes(from), orderId, selfChainId, relayChainId, to, amount);
    }

    function depositOutNative(address from, address to) external override payable whenNotPaused checkCanBridge(address(0), relayChainId) {
        require(msg.sender == from, "from only sender");
        uint amount = msg.value;
        bytes32 orderId = getOrderID(address(0), from, _addressToBytes(to), amount, relayChainId);
        IWToken(wToken).deposit{value : amount}();
        emit mapDepositOut(address(0), _addressToBytes(from), orderId, selfChainId, relayChainId, to, amount);
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


    function withdraw(address token, address payable receiver, uint256 amount) public onlyOwner checkAddress(receiver){
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
            RLPReader.RLPItem[] memory item = ls[i].toList();
            require(item.length >= 3, "log length to low");
            RLPReader.RLPItem[] memory firstItemList = item[1].toList();
            bytes[] memory topic = new bytes[](firstItemList.length);
            for (uint256 j = 0; j < firstItemList.length; j++) {
                topic[j] = firstItemList[j].toBytes();
            }
            _txLogs[i] = txLog({
            addr : item[0].toAddress(),
            topics : topic,
            data : item[2].toBytes()
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

    function changeAdmin() public {
        require(_pendingAdmin == msg.sender, "only pendingAdmin");
        emit AdminTransferred(_getAdmin(),_pendingAdmin);
        _changeAdmin(_pendingAdmin);
    }


    function pendingAdmin() external view returns(address){
        return _pendingAdmin;
    }

    function setPendingAdmin(address pendingAdmin_) public onlyOwner {
        require(pendingAdmin_ != address(0), "Ownable: pendingAdmin is the zero address");
        emit ChangePendingAdmin(_pendingAdmin, pendingAdmin_);
        _pendingAdmin = pendingAdmin_;
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}