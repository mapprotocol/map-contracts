// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "../interface/IEvent.sol";
import "./RLPReader.sol";
import "./Utils.sol";

library EvmDecoder {

    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    bytes32 constant MAP_TRANSFEROUT_TOPIC = keccak256(bytes('mapTransferOut(uint256,uint256,bytes32,bytes,bytes,bytes,uint256,bytes)'));
    bytes32 constant MAP_DEPOSITOUT_TOPIC = keccak256(bytes('mapDepositOut(uint256,uint256,bytes32,address,bytes,address,uint256)'));


    function decodeTxLogs(bytes memory logsHash)
    internal
    pure
    returns (IEvent.txLog[] memory _txLogs){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new IEvent.txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            RLPReader.RLPItem[] memory item = ls[i].toList();

            require(item.length >= 3, "log length to low");

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
        outEvent.fromChain = abi.decode(log.topics[1], (uint256));
        outEvent.toChain = abi.decode(log.topics[2], (uint256));

        (outEvent.orderId, outEvent.token, outEvent.from, outEvent.to, outEvent.amount, outEvent.toChainToken)
        = abi.decode(log.data, (bytes32, bytes, bytes, bytes, uint256, bytes));
    }

    function decodeDepositOutLog(IEvent.txLog memory log)
    internal
    pure
    returns (bytes memory executorId, IEvent.depositOutEvent memory depositEvent){
        executorId = Utils.toBytes(log.addr);

        depositEvent.fromChain = abi.decode(log.topics[1], (uint256));
        depositEvent.toChain = abi.decode(log.topics[2], (uint256));

        address token;
        address toAddress;
        (depositEvent.orderId, token, depositEvent.from, toAddress, depositEvent.amount)
        = abi.decode(log.data, (bytes32, address, bytes, address, uint256));

        depositEvent.token = Utils.toBytes(token);
        depositEvent.to = Utils.toBytes(toAddress);

    }
}
