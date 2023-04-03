// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

interface IMPTVerify {
    function verifyTrieProof(
        bytes32 root,
        bytes memory key,
        bytes[] memory proof,
        bytes memory node
    ) external pure returns (bool);
}
