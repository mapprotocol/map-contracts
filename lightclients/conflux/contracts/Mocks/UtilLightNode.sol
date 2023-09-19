// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "../lib/Types.sol";
import "../LedgerInfo.sol";

contract UtilLightNode {
    using RLPReader for RLPReader.RLPItem;


    function getHeaderBytes(Types.BlockHeader memory header) external view returns(bytes memory){
        return Types.encodeBlockHeader(header);
    }

    function getBytes(LedgerInfoLib.LedgerInfoWithSignatures memory ledgerInfo) external view returns(bytes memory) {
        return abi.encode(ledgerInfo);
    }


}
