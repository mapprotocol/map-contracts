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

    mapping(address => address) public sourceCorrespond;

    event mapTransferExecute(address indexed from, uint256 indexed fromChain, uint256 indexed toChain);

    constructor(address _wToken, address _managerAddress)
    {
        wToken = _wToken;
        lightClientManager = ILightClientManager(_managerAddress);
    }

    function transferOutToken(address _token, bytes memory _to, uint256 _amount, uint256 _toChain) external override {
        require(IERC20(_token).balanceOf(msg.sender) >= _amount, "balance too low");

        TransferHelper.safeTransferFrom(_token, msg.sender, address(this), _amount);
        _transferOut(_token, msg.sender, _to, _amount, _toChain);
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
                _transferIn(_chainId, outEvent);
            }
        }

        emit mapTransferExecute(msg.sender, _chainId, selfChainId);
    }

    function regToken(address sourceMapToken, address mapToken)
    external
    {
        sourceCorrespond[sourceMapToken] = mapToken;
    }


    function _getOrderId(address _token, address _from, bytes memory _to, uint256 _amount, uint256 _toChain) internal returns (bytes32){
        return keccak256(abi.encodePacked(nonce++, _from, _to, _token, _amount, selfChainId, _toChain));
    }

    function _transferOut(address _token, address _from, bytes memory _to, uint256 _amount, uint256 _toChain) internal {

        bytes memory toToken = Utils.toBytes(sourceCorrespond[_token]);

        bytes32 orderId = _getOrderId(_token, _from, _to, _amount, _toChain);
        emit mapTransferOut(Utils.toBytes(_token), Utils.toBytes(_from), orderId, selfChainId, _toChain, _to, _amount, toToken);
    }

    function _transferIn(uint256 _chainId, IEvent.transferOutEvent memory _outEvent)
    internal {

        address tokenB = Utils.fromBytes(_outEvent.token);

        address token =  sourceCorrespond[tokenB];

        uint256 mapOutAmount = _outEvent.amount;

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
    }

}
