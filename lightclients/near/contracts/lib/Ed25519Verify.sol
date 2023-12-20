// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

import "./Utils.sol";

library Ed25519Verify {
    address constant ED_25519_PRECOMPILE = 0x00000000000000000000000000000000000000f3;

    function checkBlockProducerSignatureInHead(
        bytes32 key,
        bytes32 r,
        bytes32 s,
        bytes32 nextHash,
        uint64 blockHeight
    ) internal view returns (bool) {
        unchecked {
            bytes memory message = abi.encodePacked(uint8(0), nextHash, Utils.swapBytes8(blockHeight + 2), bytes23(0));

            return check(key, r, s, message);
        }
    }

    function check(bytes32 k, bytes32 r, bytes32 s, bytes memory message) public view returns (bool) {
        bytes memory input = abi.encodePacked(k, r, s, message);
        require(input.length >= 96, "invalid-input-size");
        (bool success, bytes memory data) = ED_25519_PRECOMPILE.staticcall(input);
        require(success, "ed25519PreCompile call fail");

        return abi.decode(data, (bool));
    }
}
