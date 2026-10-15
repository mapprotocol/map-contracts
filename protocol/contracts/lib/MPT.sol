// SPDX-License-Identifier: MIT

/*
 * @title MerklePatriciaVerifier
 * @author Sam Mayo (sammayo888@gmail.com)
 *
 * @dev Library for verifing merkle patricia proofs.
 */

pragma solidity ^0.8.0;

import {RLPReader} from "./RLPReader.sol";

library MPT {
    /*
     * @dev Verifies a merkle patricia proof.
     * @param value The terminating value in the trie.
     * @param encodedPath The path in the trie leading to value.
     * @param rlpParentNodes The rlp encoded stack of nodes.
     * @param root The root hash of the trie.
     * @return The boolean validity of the proof.
     */
    function verify(
        bytes32 value,
        bytes memory encodedPath,
        bytes[] memory rlpParentNodes,
        bytes32 root
    ) internal pure returns (bool) {
        //  RLPReader.RLPItem memory item = RLPReader.toRlpItem(rlpParentNodes);
        //  RLPReader.RLPItem[] memory parentNodes = RLPReader.toList(item);

        bytes memory currentNode;
        RLPReader.RLPItem[] memory currentNodeList;

        bytes32 nodeKey = root;
        uint256 pathPtr = 0;
        bytes memory path = encodedPath; //_getNibbleArray(encodedPath);
        if (path.length == 0) {
            return false;
        }

        for (uint256 i = 0; i < rlpParentNodes.length; i++) {
            if (pathPtr > path.length) {
                return false;
            }

            currentNode = rlpParentNodes[i];
            if (nodeKey != keccak256(currentNode)) {
                return false;
            }
            currentNodeList = RLPReader.toList(RLPReader.toRlpItem(rlpParentNodes[i]));

            if (currentNodeList.length == 17) {
                if (pathPtr == path.length) {
                    //if (keccak256(RLPReader.toBytes(currentNodeList[16])) == keccak256(value)) {
                    if (RLPReader.payloadKeccak256(currentNodeList[16]) == value) {
                        return true;
                    } else {
                        return false;
                    }
                }

                uint8 nextPathNibble = uint8(path[pathPtr]);
                if (nextPathNibble >= 16) {
                    // only nibbles 0..15 select a child, slot 16 holds the branch value
                    return false;
                }
                nodeKey = bytes32(RLPReader.toUintStrict(currentNodeList[nextPathNibble]));
                pathPtr += 1;
            } else if (currentNodeList.length == 2) {
                bytes memory encodedPartialPath = RLPReader.toBytes(currentNodeList[0]);
                if (encodedPartialPath.length == 0) {
                    return false;
                }
                // HP (hex-prefix) flag nibble: 0/1 = extension node, 2/3 = leaf node.
                // Without this check, a 2-item node is treated as a leaf purely because
                // the path pointer happens to reach the end, even when the node is
                // actually an extension node whose second item is a child node hash
                // rather than the terminal value.
                bool isLeafNode = (uint8(encodedPartialPath[0]) >> 4) >= 2;

                (bool matched, uint256 traversed) = _nibblesToTraverse(encodedPartialPath, path, pathPtr);
                if (!matched) {
                    // the node path diverges from (or runs past the end of) the requested path
                    return false;
                }
                if (pathPtr + traversed == path.length) {
                    if (!isLeafNode) {
                        // An extension node can never be the terminal node of a valid
                        // proof path; reject instead of treating its child hash as value.
                        return false;
                    }
                    //leaf node
                    //if (keccak256(RLPReader.toBytes(currentNodeList[1])) == keccak256(value)) {
                    if (RLPReader.payloadKeccak256(currentNodeList[1]) == value) {
                        return true;
                    } else {
                        return false;
                    }
                }

                if (isLeafNode) {
                    // Leaf node whose path does not fully match the requested path.
                    return false;
                }

                //extension node
                if (traversed == 0) {
                    return false;
                }

                pathPtr += traversed;
                nodeKey = bytes32(RLPReader.toUintStrict(currentNodeList[1]));
            } else {
                return false;
            }
        }

        return false;
    }

    function _nibblesToTraverse(
        bytes memory encodedPartialPath,
        bytes memory path,
        uint256 pathPtr
    ) private pure returns (bool matched, uint256 len) {
        // encodedPartialPath has elements that are each two hex characters (1 byte), but partialPath
        // and slicedPath have elements that are each one hex character (1 nibble)
        bytes memory partialPath = _getNibbleArray(encodedPartialPath);

        // pathPtr counts nibbles in path, partialPath.length is a number of nibbles
        if (pathPtr + partialPath.length > path.length) {
            return (false, 0);
        }

        for (uint256 i = 0; i < partialPath.length; i++) {
            if (partialPath[i] != path[pathPtr + i]) {
                return (false, 0);
            }
        }
        return (true, partialPath.length);
    }

    // bytes b must be hp encoded
    function _getNibbleArray(bytes memory b) private pure returns (bytes memory) {
        bytes memory nibbles;
        if (b.length > 0) {
            uint8 offset;
            uint8 hpNibble = uint8(_getNthNibbleOfBytes(0, b));
            if (hpNibble == 1 || hpNibble == 3) {
                nibbles = new bytes(b.length * 2 - 1);
                bytes1 oddNibble = _getNthNibbleOfBytes(1, b);
                nibbles[0] = oddNibble;
                offset = 1;
            } else {
                nibbles = new bytes(b.length * 2 - 2);
                offset = 0;
            }
            //console.logUint(offset);
            for (uint256 i = offset; i < nibbles.length; i++) {
                nibbles[i] = _getNthNibbleOfBytes(i - offset + 2, b);
            }
        }
        return nibbles;
    }

    function _getNthNibbleOfBytes(uint256 n, bytes memory str) private pure returns (bytes1) {
        return bytes1(n % 2 == 0 ? uint8(str[n / 2]) / 0x10 : uint8(str[n / 2]) % 0x10);
    }
}
