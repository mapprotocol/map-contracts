// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

// import "../lib/MPT.sol";

interface IMPTVerify {
    function verifyTrieProof(
        bytes32 root,
        bytes memory key,
        bytes[] memory proof,
        bytes memory node
    ) external pure returns (bool);
}
