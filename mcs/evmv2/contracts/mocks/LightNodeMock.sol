// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

contract LightNode {
    function verifyProofData(bytes memory _receiptProof)
    external
    returns (bool success, string memory message, bytes memory logs){

        return(true,"success",_receiptProof);
    }
}
