// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.7;

import "./Types.sol";
import "./RLPEncode.sol";

library Helper {
    uint256 public constant SLOTS_PER_EPOCH = 32;
    uint256 public constant EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;

    function compute_epoch_at_slot(uint256 _slot) internal pure returns (uint256) {
        return _slot / SLOTS_PER_EPOCH;
    }

    function compute_sync_committee_period(uint256 _slot) internal pure returns (uint256){
        return compute_epoch_at_slot(_slot) / EPOCHS_PER_SYNC_COMMITTEE_PERIOD;
    }

    function getBlockHash(Types.BlockHeader memory _header)
    internal
    pure
    returns (bytes32)
    {
        bytes[] memory list = new bytes[](16);
        list[0] = RLPEncode.encodeBytes(_header.parentHash);
        list[1] = RLPEncode.encodeBytes(_header.sha3Uncles);
        list[2] = RLPEncode.encodeAddress(_header.miner);
        list[3] = RLPEncode.encodeBytes(_header.stateRoot);
        list[4] = RLPEncode.encodeBytes(_header.transactionsRoot);
        list[5] = RLPEncode.encodeBytes(_header.receiptsRoot);
        list[6] = RLPEncode.encodeBytes(_header.logsBloom);
        list[7] = RLPEncode.encodeUint(_header.difficulty);
        list[8] = RLPEncode.encodeUint(_header.number);
        list[9] = RLPEncode.encodeUint(_header.gasLimit);
        list[10] = RLPEncode.encodeUint(_header.gasUsed);
        list[11] = RLPEncode.encodeUint(_header.timestamp);
        list[12] = RLPEncode.encodeBytes(_header.extraData);
        list[13] = RLPEncode.encodeBytes(_header.mixHash);
        list[14] = RLPEncode.encodeBytes(_header.nonce);
        list[15] = RLPEncode.encodeUint(_header.baseFeePerGas);
        return keccak256(RLPEncode.encodeList(list));
    }

    function encodeReceipt(Types.TxReceipt memory _txReceipt)
    internal
    pure
    returns (bytes memory output)
    {
        bytes[] memory list = new bytes[](4);
        list[0] = RLPEncode.encodeBytes(_txReceipt.postStateOrStatus);
        list[1] = RLPEncode.encodeUint(_txReceipt.cumulativeGasUsed);
        list[2] = RLPEncode.encodeBytes(_txReceipt.bloom);
        bytes[] memory listLog = new bytes[](_txReceipt.logs.length);
        bytes[] memory loglist = new bytes[](3);
        for (uint256 j = 0; j < _txReceipt.logs.length; j++) {
            loglist[0] = RLPEncode.encodeAddress(_txReceipt.logs[j].addr);
            bytes[] memory loglist1 = new bytes[](
                _txReceipt.logs[j].topics.length
            );
            for (uint256 i = 0; i < _txReceipt.logs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes(
                    _txReceipt.logs[j].topics[i]
                );
            }
            loglist[1] = RLPEncode.encodeList(loglist1);
            loglist[2] = RLPEncode.encodeBytes(_txReceipt.logs[j].data);
            bytes memory logBytes = RLPEncode.encodeList(loglist);
            listLog[j] = logBytes;
        }
        list[3] = RLPEncode.encodeList(listLog);
        if (_txReceipt.receiptType == 0) {
            output = RLPEncode.encodeList(list);
        } else {
            bytes memory tempType = abi.encode(_txReceipt.receiptType);
            bytes1 tip = tempType[31];
            bytes memory temp = RLPEncode.encodeList(list);
            output = abi.encodePacked(tip, temp);
        }
    }

    function bytesToBytes32(bytes memory _b)
    internal
    pure
    returns (bytes32 part) {
        require(_b.length > 31, "invalid bytes length");

        assembly {
            part := mload(add(_b, 32))
        }
    }

    function getBytesSlice(bytes memory _b, uint256 _start, uint256 _length)
    internal
    pure
    returns (bytes memory) {
        require(_b.length > _start + _length - 1, "invalid bytes length");
        bytes memory out = new bytes(_length);

        for (uint256 i = 0; i < _length; i++) {
            out[i] = _b[_start + i];
        }

        return out;
    }
}