// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./ILightNodePoint.sol";

interface ILightNode is ILightNodePoint{

    //Verify the validity of the transaction according to the header, receipt, and aggPk
    //The interface will be updated later to return logs
    function verifyProofData(receiptProof memory _receiptProof) external returns (bool success, string memory message);

    //Validate headers and update validation members
    function updateBlockHeader(blockHeader memory bh, G2 memory aggPk) external;

    //Initialize the first validator
    function initialize(uint256 _threshold, address[] memory validaters, G1[] memory _pairKeys, uint256[] memory _weights,uint epoch, uint epochSize) external;
}