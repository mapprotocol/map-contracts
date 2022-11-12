// SPDX-License-Identifier: MIT


pragma solidity 0.8.7;

import "../interface/IEvent.sol";
import "./RLPReader.sol";
import "./Utils.sol";

library NearDecoder {

    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    bytes32 constant NEAR_TRANSFEROUT = 0x4e87426fdd31a6df84975ed344b2c3fbd45109085f1557dff1156b300f135df8;
    bytes32 constant NEAR_DEPOSITOUT = 0x3ad224e3e42a516df08d1fca74990eac30205afb5287a46132a6975ce0b2cede;

    function decodeNearLog(bytes memory _logs)
    internal
    pure
    returns (bytes memory executorId, IEvent.transferOutEvent memory outEvent){
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

        outEvent = IEvent.transferOutEvent({
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
    pure
    returns (bytes memory executorId, IEvent.depositOutEvent memory outEvent){
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

        outEvent = IEvent.depositOutEvent({
        token : logList[0].toBytes(),
        from : logList[1].toBytes(),
        orderId : bytes32(logList[2].toBytes()),

        fromChain : logList[3].toUint(),
        toChain : logList[4].toUint(),
        to : logList[5].toBytes(),
        amount : logList[6].toUint()
        });

    }
}
