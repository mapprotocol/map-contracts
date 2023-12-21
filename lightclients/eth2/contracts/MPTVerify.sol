// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";
import "@mapprotocol/protocol/contracts/lib/MPT.sol";

contract MPTVerify is IMPTVerify {
    function verifyTrieProof(
        bytes32 root,
        bytes memory key,
        bytes[] memory proof,
        bytes memory node
    ) external pure override returns (bool) {
        return MPT.verify(node, key, proof, root);
    }
}
