// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "./Bytes.sol";

library ProofLib {
    using Bytes for Bytes.Builder;

    bytes32 private constant EMPTY_KECCAK = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

    // full bytes recoverable
    struct NibblePath {
        bytes32 nibbles;    // use fixed array to avoid stack too deep
        uint256 start;      // inclusive
        uint256 end;        // exclusive
    }

    function _newNibblePath(bytes memory key) private pure returns (NibblePath memory) {
        require(key.length <= 16, "ProofLib: key length too long");

        bytes memory nibbles = new bytes(2 * key.length);

        for (uint256 i = 0; i < key.length; i++) {
            nibbles[2 * i] = key[i] >> 4;
            nibbles[2 * i + 1] = key[i] & 0x0F;
        }

        bytes32 nibbles32;
        assembly {
            nibbles32 := mload(add(nibbles, 32))
        }

        return NibblePath(nibbles32, 0, nibbles.length);
    }

    function _pathLength(NibblePath memory path) private pure returns (uint256) {
        return path.end - path.start;
    }

    function _pathWithoutFirstNibble(NibblePath memory path) private pure returns (bool) {
        return path.start % 2 != 0;
    }

    function _toFullBytes(NibblePath memory path) private pure returns (bytes memory) {
        bool withoutFirstNibble = _pathWithoutFirstNibble(path);
        uint256 start = withoutFirstNibble ? path.start - 1 : path.start;
        uint256 end = path.end % 2 == 1 ? path.end + 1 : path.end;

        bytes memory result = new bytes((end - start) / 2);
        uint256 offset = 0;

        for (uint256 i = start; i < end; i += 2) {
            result[offset] = bytes1(uint8(path.nibbles[i]) << 4 + uint8(path.nibbles[i + 1]));
            offset++;
        }

        if (withoutFirstNibble) {
            result[0] = result[0] & 0x0F;
        }

        return result;
    }

    function _trimPrefix(NibblePath memory path, NibblePath memory prefix) private pure returns (NibblePath memory remain, bool ok) {
        uint256 prefixLen = _pathLength(prefix);
        if (prefixLen == 0) {
            return (path, true);
        }

        for (uint256 i = 0; i < prefixLen; i++) {
            if (path.start + i >= path.end) {
                return (remain, false);
            }

            if (path.nibbles[path.start + i] != prefix.nibbles[prefix.start + i]) {
                return (remain, false);
            }
        }

        remain = NibblePath(path.nibbles, path.start + prefixLen, path.end);

        return (remain, true);
    }

    function _asChild(NibblePath memory path) private pure returns (bytes1 index, NibblePath memory child, bool ok) {
        if (_pathLength(path) == 0) {
            return (0, child, false);
        }

        index = path.nibbles[path.start];
        child = NibblePath(path.nibbles, path.start + 1, path.end);
        ok = true;
    }

    function _computePathMerkle(NibblePath memory path, bytes32 nodeMerkle) private pure returns (bytes32) {
        if (_pathLength(path) == 0) {
            return nodeMerkle;
        }

        bytes memory fullBytes = _toFullBytes(path);

        Bytes.Builder memory builder = Bytes.newBuilder(1 + fullBytes.length + 32);

        uint256 pathInfo = 128;
        if (_pathWithoutFirstNibble(path)) {
            pathInfo += 64;
        }
        pathInfo += _pathLength(path) % 63;
        builder.appendUint8(uint8(pathInfo));

        builder.appendBytes(fullBytes);
        builder.appendBytes32(nodeMerkle);

        return keccak256(builder.seal());
    }

    struct ProofNode {
        NibblePath path;
        bytes32[16] children;
        bytes value;
    }

    function _computeMerkle(ProofNode memory node) private pure returns (bytes32) {
        bytes32 nodeMerkle = _computeNodeMerkle(node.children, node.value);
        return _computePathMerkle(node.path, nodeMerkle);
    }

    function _computeNodeMerkle(bytes32[16] memory children, bytes memory value) private pure returns (bytes32) {
        uint256 valueBufLen = value.length == 0 ? 0 : value.length + 1;
        Bytes.Builder memory builder = Bytes.newBuilder(1 + 16 * 32 + valueBufLen);

        builder.appendUint8(uint8(bytes1('n')));

        // children
        for (uint256 i = 0; i < 16; i++) {
            builder.appendBytes32(children[i]);
        }

        // value
        if (value.length > 0) {
            builder.appendUint8(uint8(bytes1('v')));
            builder.appendBytes(value);
        }

        return keccak256(builder.seal());
    }

    function Prove(
        bytes32 root,
        bytes memory key,
        bytes32 valueHash,
        ProofNode[] memory nodes
    ) internal pure returns (bool) {
        require(key.length > 0, "ProofLib: key is empty");
        require(nodes.length > 0, "ProofLib: proof node is empty");

        NibblePath memory path = _newNibblePath(key);
        bytes32 expectedHash = root;

        uint256 nibblesLen = 0;
        for (uint256 i = 0; i < nodes.length; i++) {
            nibblesLen += _pathLength(nodes[i].path);
            if (nodes[i].children[0] != EMPTY_KECCAK) {
                nibblesLen++;
            }
        }

        if (nibblesLen % 2 != 0) {
            require(_pathLength(path) == nibblesLen + 1, "ProofLib: key length mismatch");
            require(path.nibbles[0] == bytes1(0), "ProofLib: first nibble of key should be 0");
            path.start++;
        } else {
            require(_pathLength(path) == nibblesLen, "ProofLib: key length mismatch");
        }

        for (uint256 i = 0; i < nodes.length; i++) {
            bytes32 merkle = _computeMerkle(nodes[i]);
            require(merkle == expectedHash, "ProofLib: proof node hash mismatch");

            (NibblePath memory remain, bool ok1) = _trimPrefix(path, nodes[i].path);
            require(ok1, "ProofLib: prefix not in common");

            if (nodes[i].children[0] == EMPTY_KECCAK) {
                // leaf ndoe
                require(_pathLength(remain) == 0, "ProofLib: key length mismatch for leaf node");
                return keccak256(nodes[i].value) == valueHash;
            } else {
                // branch node
                (bytes1 childIndex, NibblePath memory childPath, bool ok2) = _asChild(remain);
                require(ok2, "ProofLib: no child for branch node");

                path = childPath;
                expectedHash = nodes[i].children[uint256(uint8(childIndex))];
            }
        }

        revert("ProofLib: invalid key length");
    }

}
