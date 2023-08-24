// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "./ILightNodePoint.sol";

interface ILightNode is ILightNodePoint{
    event UpdateBlockHeader(address indexed account, uint256 indexed blockHeight);

    //Verify the validity of the transaction according to the header, receipt, and aggPk
    //The interface will be updated later to return logs
    function verifyProofData(bytes memory _receiptProof)
    external
    returns (bool success, string memory message,bytes memory logsHash);

    //Validate headers and update validation members
    function updateBlockHeader(bytes memory _blockHeaders) external;

    //Initialize the first validator
    function initialize(
        address[]  memory _validators,
        uint256 _headerHeight,
        address _verifyTool
    ) external;

    function verifiableHeaderRange() external view returns (uint256, uint256);
}