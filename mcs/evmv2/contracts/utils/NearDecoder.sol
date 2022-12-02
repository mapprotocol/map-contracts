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

    function decodeNearLog(bytes memory logsHash)
    internal
    view
    returns (bytes memory executorId, IEvent.transferOutEvent[] memory _outEvents){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();

        require(ls.length >= 2, "logsHash length to low");

        executorId = ls[0].toBytes();

        bytes[] memory logs = new bytes[](ls[1].toList().length);
        for (uint256 i = 0; i < ls[1].toList().length; i++) {
            logs[i] = ls[1].toList()[i].toBytes();
        }
        bytes memory log;
        for (uint256 i = 0; i < logs.length; i++) {

            (bytes memory temp) = splitExtra(logs[i]);
            if (keccak256(temp) == nearTransferOut) {
                log = hexStrToBytes(logs[i]);
                RLPReader.RLPItem[] memory logList = log.toRlpItem().toList();
                require(logList.length >= 8, "logsHash length to low");
                IEvent.transferOutEvent memory _outEvent = transferOutEvent({
                token : logList[0].toBytes(),
                from : logList[1].toBytes(),
                order_id : bytes32(logList[2].toBytes()),
                from_chain : logList[3].toUint(),
                to_chain : logList[4].toUint(),
                to : logList[5].toBytes(),
                amount : logList[6].toUint(),
                to_chain_token : logList[7].toBytes()
                });
                _outEvents[i] = _outEvent;
            }
        }
    }

    function decodeNearDepositLog(bytes memory logsHash)
    public
    view
    returns (bytes memory executorId, IEvent.depositOutEvent[] memory _outEvents){
        RLPReader.RLPItem[] memory ls = logsHash.toRlpItem().toList();
        require(ls.length >= 2, "logsHash length to low");

        executorId = ls[0].toBytes();

        bytes[] memory logs = new bytes[](ls[1].toList().length);
        for (uint256 i = 0; i < ls[1].toList().length; i++) {

            logs[i] = ls[1].toList()[i].toBytes();

        }
        bytes memory log;
        for (uint256 i = 0; i < logs.length; i++) {

            (bytes memory temp) = splitExtra(logs[i]);
            if (keccak256(temp) == nearDepositOut) {
                log = hexStrToBytes(logs[i]);
                RLPReader.RLPItem[] memory logList = log.toRlpItem().toList();

                require(logList.length >= 7, "logsHash length to low");

                IEvent.depositOutEvent memory _outEvent = nearDepositOutEvent({
                token : logList[0].toBytes(),
                from : logList[1].toBytes(),
                order_id : logList[2].toBytes(),
                from_chain : logList[3].toUint(),
                to_chain : logList[4].toUint(),
                to : logList[5].toBytes(),
                amount : logList[6].toUint()
                });
                _outEvents[i] = _outEvent;
            }
        }
    }
}
