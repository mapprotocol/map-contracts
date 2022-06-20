// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {

    function verifyProofData(bytes memory _receiptProof) external returns (bool success, bytes memory logs);

    function updateBlockHeader(bytes memory _blackHeader) external;
}