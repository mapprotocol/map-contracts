// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./ILogs.sol";

interface ILightNode is ILogs{
    function verifyProofData(bytes memory _receiptProof) external returns (bool success, string memory message, bytes memory logs);
}