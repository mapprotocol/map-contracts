// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "./Bytes.sol";

/**
 * @dev A simple BCS encoding library.
 */
library BCS {
    using Bytes for Bytes.Builder;

    uint256 public constant SIZE_BYTES32 = 32;
    uint256 public constant SIZE_UINT64 = 8;
    uint256 public constant SIZE_OPTION = 1;

    uint256 private constant LEN_MAX = (1 << 31) - 1;

    function sizeLength(uint256 len) internal pure returns (uint256) {
        if (len < 0x80) return 1;
        if (len < 0x4000) return 2;
        if (len < 0x200000) return 3;
        if (len < 0x10000000) return 4;
        require(len <= LEN_MAX, "BCS: length too large");
        return 5;
    }

    function sizeBytes(bytes memory val) internal pure returns (uint256) {
        return sizeLength(val.length) + val.length;
    }

    function encodeBytes32(Bytes.Builder memory builder, bytes32 val) internal pure {
        builder.appendBytes32(val);
    }

    function encodeLength(Bytes.Builder memory builder, uint256 length) internal pure {
        while (length >= 0x80) {
            builder.appendUint8(uint8((length & 0x7F) | 0x80));
            length >>= 7;
        }

        builder.appendUint8(uint8(length));
    }

    function encodeBytes(Bytes.Builder memory builder, bytes memory val) internal pure {
        encodeLength(builder, val.length);
        builder.appendBytes(val);
    }

    function encodeUint64(Bytes.Builder memory builder, uint64 val) internal pure {
        // encode in little endian
        for (uint256 i = 0; i < 8; i++) {
            builder.appendUint8(uint8(val & 0xFF));
            val >>= 8;
        }
    }

    function encodeOption(Bytes.Builder memory builder, bool some) internal pure {
        if (some) {
            builder.appendUint8(1);
        } else {
            builder.appendUint8(0);
        }
    }

}
