// SPDX-License-Identifier: MIT

pragma solidity 0.8.0;

import "./interface/IMPTVerify.sol";
import "./lib/MPT.sol";

contract MPTVerify is IMPTVerify {
    function verifyTrieProof(
        bytes32 root,
        bytes memory key,
        bytes[] memory proof,
        bytes memory node
    ) external pure override returns (bool) {
        MPT.MerkleProof memory mp = MPT.MerkleProof({
            expectedRoot: root,
            key: key,
            proof: proof,
            keyIndex: 0,
            proofIndex: 0,
            expectedValue: node
        });
        return MPT.verifyTrieProof(mp);
    }
}
