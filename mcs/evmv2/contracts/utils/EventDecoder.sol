// SPDX-License-Identifier: MIT


pragma solidity >=0.7.1;

import "./RLPReader.sol";
import "./Utils.sol";

library EventDecoder {

    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    bytes32 constant MAP_TRANSFEROUT_TOPIC = keccak256(bytes('mapTransferOut(bytes,bytes,bytes32,uint256,uint256,bytes,uint256,bytes)'));
    bytes32 constant MAP_DEPOSITOUT_TOPIC = keccak256(bytes('mapDepositOut(address,bytes,bytes32,address,uint256)'));
    bytes32 constant NEAR_TRANSFEROUT = 0x4e87426fdd31a6df84975ed344b2c3fbd45109085f1557dff1156b300f135df8;
    bytes32 constant NEAR_DEPOSITOUT = 0x3ad224e3e42a516df08d1fca74990eac30205afb5287a46132a6975ce0b2cede;

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

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }

    function decodeTxLog(bytes memory logsHash)
    internal
    pure
    returns (txLog[] memory _txLogs){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        _txLogs = new txLog[](ls.length);
        for (uint256 i = 0; i < ls.length; i++) {
            RLPReader.RLPItem[] memory item = ls[i].toList();
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

    function decodeTxLog(txLog memory log)
    internal
    pure
    returns (bytes memory executorId, transferOutEvent memory outEvent){
        executorId = Utils.toBytes(log.addr);
        (outEvent.token, outEvent.from, outEvent.orderId, outEvent.fromChain,
        outEvent.toChain, outEvent.to, outEvent.amount,)
        = abi.decode(log.data, (bytes, bytes, bytes32, uint256, uint256, bytes, uint256, bytes));
    }

    function decodeNearLog(bytes memory _logs)
    internal
    view
    returns (bytes memory executorId, transferOutEvent memory outEvent){
        RLPReader.RLPItem[] memory ls = _logs.toRlpItem().toList();

        executorId = ls[0].toBytes();

        bytes[] memory logs = new bytes[](ls[1].toList().length);
        for (uint256 i = 0; i < ls[1].toList().length; i++) {

            logs[i] = ls[1].toList()[i].toBytes();

        }
        bytes memory log;
        for (uint256 i = 0; i < logs.length; i++) {
            (bytes memory temp) = Utils.splitExtra(logs[i]);
            if (keccak256(temp) == NEAR_TRANSFEROUT) {
                log = Utils.hexStrToBytes(logs[i]);
            }
        }

        RLPReader.RLPItem[] memory logList = log.toRlpItem().toList();

        outEvent = transferOutEvent({
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

    function decodeNearDepositLog(bytes memory _logs)
    public
    view
    returns (bytes memory executorId, depositOutEvent memory outEvent){
        RLPReader.RLPItem[] memory ls = _logs.toRlpItem().toList();

        executorId = ls[0].toBytes();

        bytes[] memory logs = new bytes[](ls[1].toList().length);
        for (uint256 i = 0; i < ls[1].toList().length; i++) {

            logs[i] = ls[1].toList()[i].toBytes();

        }
        bytes memory log;
        for (uint256 i = 0; i < logs.length; i++) {
            (bytes memory temp) = Utils.splitExtra(logs[i]);
            if (keccak256(temp) == NEAR_DEPOSITOUT) {
                log = Utils.hexStrToBytes(logs[i]);
            }
        }

        RLPReader.RLPItem[] memory logList = log.toRlpItem().toList();

        outEvent = depositOutEvent({
        token : logList[0].toBytes(),
        from : logList[1].toBytes(),
        orderId : bytes32(logList[2].toBytes()),
        to : logList[3].toBytes(),
        amount : logList[4].toUint()
        });

    }

}
