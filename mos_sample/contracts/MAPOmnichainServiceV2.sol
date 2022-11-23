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
import "./utils/TransferHelper.sol";
import "./interface/IMOSV2.sol";
import "./interface/ILightNode.sol";
import "./utils/RLPReader.sol";
import "./utils/Utils.sol";
import "./utils/EvmDecoder.sol";


contract MAPOmnichainServiceV2 is ReentrancyGuard, Initializable, IMOSV2 {
    using SafeMath for uint;
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint public immutable selfChainId = block.chainid;
    uint public nonce;
    address public wToken;          // native wrapped token
    address public relayContract;
    uint256 public relayChainId;
    ILightNode public lightNode;

    event mapTransferExecute(address indexed from, uint256 indexed fromChain, uint256 indexed toChain);

    function initialize(address _wToken, address _lightNode)
    public initializer {
        wToken = _wToken;
        lightNode = ILightNode(_lightNode);
    }


    receive() external payable {
        require(msg.sender == wToken, "only wToken");
    }

    function emergencyWithdraw(address _token, address payable _receiver, uint256 _amount) external {
        if (_token == wToken) {
            TransferHelper.safeWithdraw(wToken, _amount);
            TransferHelper.safeTransferETH(_receiver, _amount);
        } else {
            IERC20(_token).transfer(_receiver, _amount);
        }
    }

    function transferOutToken(address _token, bytes memory _to, uint256 _amount, uint256 _toChain) external override nonReentrant
    {

        require(IERC20(_token).balanceOf(msg.sender) >= _amount, "balance too low");

        TransferHelper.safeTransferFrom(_token, msg.sender, address(this), _amount);

        bytes32 orderId = _getOrderID(_token, msg.sender, _to, _amount, _toChain);
        emit mapTransferOut(Utils.toBytes(_token), Utils.toBytes(msg.sender), orderId, selfChainId, _toChain, _to, _amount, Utils.toBytes(address(0)));
    }

    function transferOutNative(bytes memory _to, uint _toChain) external override payable nonReentrant
    {
        require(_toChain != selfChainId, "only other chain");
        uint amount = msg.value;
        require(amount > 0, "balance is zero");
        IWToken(wToken).deposit{value : amount}();
        bytes32 orderId = _getOrderID(wToken, msg.sender, _to, amount, _toChain);
        emit mapTransferOut(Utils.toBytes(wToken), Utils.toBytes(msg.sender), orderId, selfChainId, _toChain, _to, amount, Utils.toBytes(address(0)));
    }


    function transferIn(uint256 _chainId, bytes memory _receiptProof) external nonReentrant {
        (bool sucess, string memory message, bytes memory logArray) = lightNode.verifyProofData(_receiptProof);
        require(sucess, message);
        IEvent.txLog[] memory logs = EvmDecoder.decodeTxLogs(logArray);

        for (uint i = 0; i < logs.length; i++) {
            IEvent.txLog memory log = logs[i];
            bytes32 topic = abi.decode(log.topics[0], (bytes32));

            if (topic == EvmDecoder.MAP_TRANSFEROUT_TOPIC && relayContract == log.addr) {
                (, IEvent.transferOutEvent memory outEvent) = EvmDecoder.decodeTransferOutLog(log);
                // there might be more than on events to multi-chains
                // only process the event for this chain
                if (selfChainId == outEvent.toChain) {
                    _transferIn(outEvent);
                }
            }
        }
        emit mapTransferExecute(msg.sender, _chainId, selfChainId);
    }

    function _getOrderID(address _token, address _from, bytes memory _to, uint _amount, uint _toChain) internal returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, _from, _to, _token, _amount, selfChainId, _toChain));
    }

    function _transferIn(IEvent.transferOutEvent memory _outEvent)
    internal {
        address token = Utils.fromBytes(_outEvent.toChainToken);
        address payable toAddress = payable(Utils.fromBytes(_outEvent.to));
        uint256 amount = _outEvent.amount;
        if (token == wToken) {
            TransferHelper.safeWithdraw(wToken, amount);
            TransferHelper.safeTransferETH(toAddress, amount);
        } else {
            TransferHelper.safeTransfer(token, toAddress, amount);
        }

        emit mapTransferIn(token, _outEvent.from, _outEvent.orderId, _outEvent.fromChain, _outEvent.toChain, toAddress, amount);
    }

}