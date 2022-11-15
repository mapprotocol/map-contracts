// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "../interface/IEvent.sol";
import "./RLPReader.sol";
import "./Utils.sol";

library EvmDecoder {

    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    bytes32 constant MAP_TRANSFEROUT_TOPIC = keccak256(bytes('mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)'));
    bytes32 constant MAP_DEPOSITOUT_TOPIC = keccak256(bytes('mapDepositOut(address,bytes,bytes32,uint256,uint256,address,uint256)'));


    function decodeTxLogs(bytes memory logsHash)
    internal
    pure
    returns (IEvent.txLog[] memory _txLogs){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new IEvent.txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            RLPReader.RLPItem[] memory item = ls[i].toList();
            RLPReader.RLPItem[] memory firstItemList = item[1].toList();
            bytes[] memory topic = new bytes[](firstItemList.length);
            for (uint256 j = 0; j < firstItemList.length; j++) {
                topic[j] = firstItemList[j].toBytes();
            }
            _txLogs[i] = IEvent.txLog({
            addr : item[0].toAddress(),
            topics : topic,
            data : item[2].toBytes()
            });
        }
    }

    function decodeTransferOutLog(IEvent.txLog memory log)
    internal
    pure
    returns (bytes memory executorId, IEvent.transferOutEvent memory outEvent){
        executorId = Utils.toBytes(log.addr);
        (outEvent.token, outEvent.from, outEvent.orderId, outEvent.fromChain,
        outEvent.toChain, outEvent.to, outEvent.amount,outEvent.toChainToken)
        = abi.decode(log.data, (bytes, bytes, bytes32, uint256, uint256, bytes, uint256, bytes));
    }

    function decodeDepositOutLog(IEvent.txLog memory log)
    internal
    pure
    returns (bytes memory executorId, IEvent.depositOutEvent memory depositEvent){
        executorId = Utils.toBytes(log.addr);
        address tokenAddress =  abi.decode(log.topics[1],(address));
        depositEvent.token = Utils.toBytes(tokenAddress);
        address toAddress;
        ( depositEvent.from, depositEvent.orderId, depositEvent.fromChain,
        depositEvent.toChain,toAddress, depositEvent.amount)
        = abi.decode(log.data, (bytes, bytes32, uint256, uint256, address, uint256));

        depositEvent.to = Utils.toBytes(toAddress);

    }
}
