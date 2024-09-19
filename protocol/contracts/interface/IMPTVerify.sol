// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IMPTVerify {
    function verifyTrieProof(
        bytes32 _root,
        bytes memory _key,
        bytes[] memory _proof,
        bytes memory _node
    ) external pure returns (bool);

    function verifyTrieProof(
        bytes32 _root,
        bytes32 _value,
        bytes memory _key,
        bytes[] memory _proof
    ) external pure returns (bool);
}
