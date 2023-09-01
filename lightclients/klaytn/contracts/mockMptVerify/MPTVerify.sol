// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "../interface/IMPTVerify.sol";
import "./MPT.sol";
contract MPTVerify is IMPTVerify {
    function verifyTrieProof(
        bytes32 _root,
        bytes memory _key,
        bytes[] memory _proof,
        bytes memory _node
    ) external pure override returns (bool) {
//        MPT.MerkleProof memory mp;
//        mp.expectedRoot = _root;
//        mp.key = _key;
//        mp.proof = _proof;
//        mp.keyIndex = 0;
//        mp.proofIndex = 0;
//        mp.expectedValue = _node;

        return MPT.verify(_node,_key,_proof,_root);
    }
}
