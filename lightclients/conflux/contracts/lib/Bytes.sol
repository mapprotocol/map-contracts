// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

/**
 * @dev This is used to optimize gas cost for bytes operations.
 *
 * Note, client should make sure the buffer size is enough for append operations.
 */
library Bytes {

    uint256 private constant BYTES_HEADER_SIZE = 32;
    uint256 private constant WORD = 32;

    struct Builder {
        bytes buf;
        uint256 offset;
    }

    function newBuilder(uint256 size) internal pure returns (Builder memory) {
        return Builder(new bytes(size), 0);
    }

    function seal(Builder memory builder) internal pure returns (bytes memory) {
        require(builder.offset == builder.buf.length, "Bytes: buffer not fully filled");
        return builder.buf;
    }

    function reset(Builder memory builder) internal pure {
        builder.offset = 0;
    }

    function appendUint8(Builder memory builder, uint8 val) internal pure {
        builder.buf[builder.offset] = bytes1(val);
        builder.offset++;
    }

    function appendBytes32(Builder memory builder, bytes32 val) internal pure {
        bytes memory buf = builder.buf;
        uint256 offset = BYTES_HEADER_SIZE + builder.offset;

        assembly {
            mstore(add(buf, offset), val)
        }

        builder.offset += 32;
    }

    function appendBytes(Builder memory builder, bytes memory val) internal pure {
        memcopy(builder.buf, builder.offset, val, 0, val.length);
        builder.offset += val.length;
    }

    function appendBytes(Builder memory builder, bytes memory val, uint256 offset, uint256 len) internal pure {
        memcopy(builder.buf, builder.offset, val, offset, len);
        builder.offset += len;
    }

    function appendEmpty(Builder memory builder, uint256 n) internal pure {
        builder.offset += n;
    }

    function appendIntOSP(Builder memory builder, uint256 x, uint256 len) internal pure {
        uint256 index = builder.offset + len - 1;

        while (x > 0) {
            builder.buf[index] = bytes1(uint8(x & 0xFF)); // big endian
            index--;
            x >>= 8;
        }

        builder.offset += len;
    }

    function concat2(bytes memory b1, bytes memory b2) internal pure returns (bytes memory) {
        bytes memory result = new bytes(b1.length + b2.length);
        memcopy(result, 0, b1, 0, b1.length);
        memcopy(result, b1.length, b2, 0, b2.length);
        return result;
    }

    function copy(bytes memory b1, uint256 offset, bytes memory b2) internal pure {
        memcopy(b1, offset, b2, 0, b2.length);
    }

    function memcopy(bytes memory dst, uint256 dstOffset, bytes memory src, uint256 srcOffset, uint256 len) internal pure {
        require(srcOffset + len <= src.length, "Bytes: src out of bound");
        require(dstOffset + len <= dst.length, "Bytes: dst out of bound");

        uint256 srcPtr;
        uint256 dstPtr;
        assembly {
            srcPtr := add(src, add(BYTES_HEADER_SIZE, srcOffset))
            dstPtr := add(dst, add(BYTES_HEADER_SIZE, dstOffset))
        }

        // copy word by word
        uint256 copied = len / WORD * WORD;
        for (; len >= WORD; len -= WORD) {
            assembly {
                mstore(dstPtr, mload(srcPtr))
                srcPtr := add(srcPtr, WORD)
                dstPtr := add(dstPtr, WORD)
            }
        }

        if (len > 0) {
            _copyIncompleteWord(dst, dstOffset + copied, src, srcOffset + copied, len);
        }
    }

    function _copyIncompleteWord(bytes memory dst, uint256 dstOffset, bytes memory src, uint256 srcOffset, uint256 len) private pure {
        if (dstOffset + len >= WORD) {
            dstOffset = dstOffset + len - WORD;
            assembly {
                let srcPart := mload(add(src, add(BYTES_HEADER_SIZE, srcOffset)))
                srcPart := shr(mul(sub(WORD, len), 8), srcPart)

                let dstPtr := add(dst, add(BYTES_HEADER_SIZE, dstOffset))
                let dstPart := mload(dstPtr)
                dstPart := shr(mul(len, 8), dstPart)
                dstPart := shl(mul(len, 8), dstPart)

                mstore(dstPtr, or(srcPart, dstPart))
            }
        } else {
            uint256 tailLen = WORD - dstOffset - len;
            uint256 mask = ((1 << (len * 8)) - 1) << tailLen;
            assembly {
                let srcPart := mload(add(src, add(BYTES_HEADER_SIZE, srcOffset)))
                srcPart := shr(mul(sub(WORD, len), 8), srcPart)
                srcPart := shl(mul(tailLen, 8), srcPart)

                let dstPtr := add(dst, BYTES_HEADER_SIZE)
                let dstPart := mload(dstPtr)
                dstPart := and(dstPart, not(mask))
                mstore(dstPtr, or(srcPart, dstPart))
            }
        }
    }

}
