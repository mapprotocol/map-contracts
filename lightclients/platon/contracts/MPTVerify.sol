// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";

contract MPTVerify is IMPTVerify {
    function verifyTrieProof(
        bytes32 _root,
        bytes memory _key,
        bytes[] memory _proof,
        bytes memory _node
    ) external pure override returns (bool) {
        return MPT.verify(_node, _key, _proof, _root);
    }
}
