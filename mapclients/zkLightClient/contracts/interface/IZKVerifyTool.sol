// SPDX-License-Identifier: MIT

pragma solidity 0.8.21;

interface IZKVerifyTool {
    function verifyProof(uint[8] memory proofs, uint[] memory inputs) external view returns (bool isVerified);
}
