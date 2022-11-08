// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./ILightNodePoint.sol";

interface ILightNode is ILightNodePoint{

    //Verify the validity of the transaction according to the header, receipt, and aggPk
    //The interface will be updated later to return logs
    function verifyProofData(bytes memory _receiptProof)
    external
    returns (bool success, string memory message,bytes memory logsHash);

    //Validate headers and update validation members
    function updateBlockHeader(blockHeader memory bh, G2 memory aggPk) external;

    //Initialize the first validator
    function initialize(
        //The total weight of votes
        uint256 _threshold,
        //committee members
        address[] memory validators,
        //G1 public key corresponding to the committee member
        G1[] memory _pairKeys,
        //Weights corresponding to committee members
        uint256[] memory _weights,
        //number of committees
        uint epoch,
        //The number of blocks corresponding to each committee
        uint epochSize,
        address verifyTool)
    external;
}