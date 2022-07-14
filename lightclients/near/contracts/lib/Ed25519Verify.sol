// SPDX-License-Identifier: MIT
pragma solidity ^0.8;

import "./Utils.sol";

library Ed25519Verify {
    address constant ed25519PreCompile = 0x00000000000000000000000000000000000000f3;

    function checkBlockProducerSignatureInHead(
        bytes32 key,
        bytes32 r,
        bytes32 s,
        bytes32 nextHash,
        uint64 blockHeight
    ) internal view returns (bool) {
        unchecked {
            bytes memory message = abi.encodePacked(
                uint8(0),
                nextHash,
                Utils.swapBytes8(blockHeight + 2),
                bytes23(0)
            );

            return check(key, r, s, message);
        }
    }

    function check(
        bytes32 k,
        bytes32 r,
        bytes32 s,
        bytes memory message
    ) public view returns (bool) {
        bytes memory input = abi.encodePacked(k, r, s, message);

        (bool success, bytes memory data) = ed25519PreCompile.staticcall(input);
        require(success);

        return abi.decode(data, (bool));
    }
}
