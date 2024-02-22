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
        MPT.MerkleProof memory mp = MPT.MerkleProof({
            expectedRoot: _root,
            key: _key,
            proof: _proof,
            keyIndex: 0,
            proofIndex: 0,
            expectedValue: _node
        });
        return MPT.verifyTrieProof(mp);
    }
}
