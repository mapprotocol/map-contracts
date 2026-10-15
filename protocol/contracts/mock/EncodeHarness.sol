// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import {RLPEncode} from "../lib/RLPEncode.sol";
import {LibRLP} from "../lib/LibRLP.sol";

contract RLPEncodeHarness {
    function encodeBytes(bytes memory b) external pure returns (bytes memory) {
        return RLPEncode.encodeBytes(b);
    }

    function encodeString(string memory s) external pure returns (bytes memory) {
        return RLPEncode.encodeString(s);
    }

    function encodeUint(uint256 x) external pure returns (bytes memory) {
        return RLPEncode.encodeUint(x);
    }

    function encodeInt(int256 x) external pure returns (bytes memory) {
        return RLPEncode.encodeInt(x);
    }

    function encodeAddress(address a) external pure returns (bytes memory) {
        return RLPEncode.encodeAddress(a);
    }

    function encodeBool(bool b) external pure returns (bytes memory) {
        return RLPEncode.encodeBool(b);
    }

    function encodeList(bytes[] memory items) external pure returns (bytes memory) {
        return RLPEncode.encodeList(items);
    }

    /// @dev encodes [address, uint, address] in one call, to detect memory reuse problems in encodeAddress
    function encodeAddressesInOneCall(address a, uint256 x, address b) external pure returns (bytes memory) {
        bytes[] memory items = new bytes[](3);
        items[0] = RLPEncode.encodeAddress(a);
        items[1] = RLPEncode.encodeUint(x);
        items[2] = RLPEncode.encodeAddress(b);
        return RLPEncode.encodeList(items);
    }

    /// @dev encodeAddress after the free memory above the pointer was dirtied
    function encodeAddressAfterDirtyMemory(address a) external pure returns (bytes memory) {
        assembly {
            let m := mload(0x40)
            mstore(m, not(0))
            mstore(add(m, 0x20), not(0))
        }
        return RLPEncode.encodeAddress(a);
    }
}

contract LibRLPHarness {
    using LibRLP for LibRLP.List;

    function encodeUint(uint256 x) external pure returns (bytes memory) {
        return LibRLP.encode(x);
    }

    function encodeAddress(address x) external pure returns (bytes memory) {
        return LibRLP.encode(x);
    }

    function encodeBool(bool x) external pure returns (bytes memory) {
        return LibRLP.encode(x);
    }

    function encodeBytes(bytes memory x) external pure returns (bytes memory) {
        return LibRLP.encode(x);
    }

    function encodeUints(uint256[] memory xs) external pure returns (bytes memory) {
        LibRLP.List memory l = LibRLP.l();
        for (uint256 i = 0; i < xs.length; i++) {
            l = l.p(xs[i]);
        }
        return l.encode();
    }

    /// @dev [a, b, c, [d...]]
    function encodeMixed(uint256 a, address b, bytes memory c, uint256[] memory d) external pure returns (bytes memory) {
        LibRLP.List memory inner = LibRLP.l();
        for (uint256 i = 0; i < d.length; i++) {
            inner = inner.p(d[i]);
        }
        return LibRLP.l().p(a).p(b).p(c).p(inner).encode();
    }
}
