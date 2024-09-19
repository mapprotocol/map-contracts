// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "./interface/IMPTVerify.sol";
import "./lib/MPT.sol";

contract MPTVerify is IMPTVerify {
    function verifyTrieProof(
        bytes32 _root,
        bytes memory _key,
        bytes[] memory _proof,
        bytes memory _node
    ) external pure override returns (bool) {
        bytes32 value = keccak256(_node);
        return MPT.verify(value, _key, _proof, _root);
    }

    function verifyTrieProof(
        bytes32 _root,
        bytes32 _value,
        bytes memory _key,
        bytes[] memory _proof
    ) external pure override returns (bool) {
        return MPT.verify(_value, _key, _proof, _root);
    }
}
