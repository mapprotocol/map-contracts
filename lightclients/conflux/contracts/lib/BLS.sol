// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "./Bytes.sol";

import "@openzeppelin/contracts/utils/Strings.sol";

/**
 * @dev BLS12-381 library to verify BLS signatures.
 */
library BLS {
    using Bytes for Bytes.Builder;

    // Point G1 of -1
    bytes32 private constant G1_NEG_ONE_0 = 0x0000000000000000000000000000000017f1d3a73197d7942695638c4fa9ac0f;
    bytes32 private constant G1_NEG_ONE_1 = 0xc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb;
    bytes32 private constant G1_NEG_ONE_2 = 0x00000000000000000000000000000000114d1d6855d545a8aa7d76c8cf2e21f2;
    bytes32 private constant G1_NEG_ONE_3 = 0x67816aef1db507c96655b9d5caac42364e6f38ba0ecb751bad54dcd6b939c2ca;

    address private constant PRECOMPILE_BIG_MOD_EXP = 0x0000000000000000000000000000000000000005;
    address internal constant PRECOMPILE_BLS12_G1ADD = 0x000000000000000000000000000000000000000A;
    address private constant PRECOMPILE_BLS12_MAP_FP2_TO_G2 = 0x0000000000000000000000000000000000000012;
    address private constant PRECOMPILE_BLS12_G2ADD = 0x000000000000000000000000000000000000000d;
    address private constant PRECOMPILE_BLS12_PAIRING = 0x0000000000000000000000000000000000000010;

    /**
     * @dev Aggregate BLS public keys via BLS12_G1ADD.
     */
    function aggregatePublicKeys(bytes[] memory publicKeys) internal view returns (bytes memory) {
        require(publicKeys.length > 0, "BLS: empty public keys");

        if (publicKeys.length == 1) {
            return publicKeys[0];
        }

        Bytes.Builder memory builder = Bytes.newBuilder(256);
        _appendPublicKey(builder, publicKeys[0]);
        bytes memory buf;

        for (uint256 i = 1; i < publicKeys.length; i++) {
            _appendPublicKey(builder, publicKeys[i]);
            buf = builder.seal();
            callPrecompile(PRECOMPILE_BLS12_G1ADD, buf, buf, 0, 128);
            builder.reset();
            builder.appendEmpty(128);
        }

        builder.appendEmpty(128);
        buf = builder.seal();

        bytes memory agg = new bytes(128);
        Bytes.memcopy(agg, 0, buf, 0, 128);

        return agg;
    }

    /**
     * @dev Aggregate BLS signatures via BLS12_G2ADD.
     */
    function aggregateSignatures(bytes[] memory signatures) internal view returns (bytes memory) {
        require(signatures.length > 0, "BLS: empty signatures");

        if (signatures.length == 1) {
            return signatures[0];
        }

        Bytes.Builder memory builder = Bytes.newBuilder(512);
        _appendSignature(builder, signatures[0]);
        bytes memory buf;

        for (uint256 i = 1; i < signatures.length; i++) {
            _appendSignature(builder, signatures[i]);
            buf = builder.seal();
            callPrecompile(PRECOMPILE_BLS12_G2ADD, buf, buf, 0, 256);
            builder.reset();
            builder.appendEmpty(256);
        }

        builder.appendEmpty(256);
        buf = builder.seal();

        bytes memory agg = new bytes(256);
        Bytes.memcopy(agg, 0, buf, 64, 64);
        Bytes.memcopy(agg, 64, buf, 0, 64);
        Bytes.memcopy(agg, 128, buf, 192, 64);
        Bytes.memcopy(agg, 192, buf, 128, 64);

        return agg;
    }

    /**
     * @dev Batch verify BLS signatures.
     * @param signatures uncompressed BLS signatures.
     * @param message message to verify.
     * @param publicKeys uncompressed BLS public keys.
     */
    function batchVerify(bytes[] memory signatures, bytes memory message, bytes[] memory publicKeys) internal view returns (bool) {
        require(signatures.length == publicKeys.length, "signatures and publicKeys length mismatch");
        bytes memory aggSignature = aggregateSignatures(signatures);
        return aggregateVerify(aggSignature, message, publicKeys);
    }

    /**
     * @dev Verify aggregated BLS signature.
     * @param signature aggregated BLS signature.
     * @param message message to verify.
     * @param publicKeys uncompressed BLS public keys.
     */
    function aggregateVerify(bytes memory signature, bytes memory message, bytes[] memory publicKeys) internal view returns (bool) {
        bytes memory aggPubKey = aggregatePublicKeys(publicKeys);
        return verify(signature, message, aggPubKey);
    }

    /**
     * @dev verify BLS signature.
     * @param signature uncompressed BLS signature.
     * @param message message to verify.
     * @param publicKey uncompressed BLS public key.
     */
    function verify(bytes memory signature, bytes memory message, bytes memory publicKey) internal view returns (bool) {
        bytes memory hashedMessage = hashToCurve(message);
        return verifyHashed(signature, hashedMessage, publicKey);
    }

    function verifyHashed(bytes memory signature, bytes memory hashedMessage, bytes memory publicKey) internal view returns (bool) {
        Bytes.Builder memory builder = Bytes.newBuilder(768);

        // public key
        _appendPublicKey(builder, publicKey);

        // message
        builder.appendBytes(hashedMessage);

        // -1
        builder.appendBytes32(G1_NEG_ONE_0);
        builder.appendBytes32(G1_NEG_ONE_1);
        builder.appendBytes32(G1_NEG_ONE_2);
        builder.appendBytes32(G1_NEG_ONE_3);

        // signature
        _appendSignature(builder, signature);

        // pairing
        bytes memory output = new bytes(32);
        callPrecompile(PRECOMPILE_BLS12_PAIRING, builder.seal(), output);
        return abi.decode(output, (bool));
    }

    function hashToCurve(bytes memory message) internal view returns (bytes memory) {
        bytes memory fe = hashToField(message);

        bytes memory p = new bytes(512);
        callPrecompile(PRECOMPILE_BLS12_MAP_FP2_TO_G2, fe, 0, 128, p, 0, 256);
        callPrecompile(PRECOMPILE_BLS12_MAP_FP2_TO_G2, fe, 128, 128, p, 256, 256);

        bytes memory output = new bytes(256);
        callPrecompile(PRECOMPILE_BLS12_G2ADD, p, output);
        return output;
    }

    uint256 private constant H_IN_CHUNK_SIZE = 64;
    uint256 private constant H_OUT_CHUNK_SIZE = 32;
    uint256 private constant L = 64;
    uint256 private constant MSG_LEN = L * 2 * 2; // 256
    uint256 private constant ELL = MSG_LEN / H_OUT_CHUNK_SIZE; // 8

    bytes private constant DST_SUFFIX = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_+";
    bytes32 private constant P_0 = 0x000000000000000000000000000000001a0111ea397fe69a4b1ba7b6434bacd7;
    bytes32 private constant P_1 = 0x64774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab;

    function hashToField(bytes memory message) internal view returns (bytes memory) {
        bytes memory expanded = expandMessageXmd(message);

        Bytes.Builder memory builder = Bytes.newBuilder(32 * 3 + L + 1 + L);
        builder.appendIntOSP(L, 32);
        builder.appendIntOSP(1, 32);
        builder.appendIntOSP(L, 32);
        builder.appendEmpty(L); // placeholder for expanded message
        builder.appendUint8(1);
        builder.appendBytes32(P_0);
        builder.appendBytes32(P_1);

        for (uint256 i = 0; i < MSG_LEN; i += L) {
            _inPlaceBigMod(builder, expanded, i);
        }

        return expanded;
    }

    function expandMessageXmd(bytes memory message) internal pure returns (bytes memory) {
        Bytes.Builder memory b = Bytes.newBuilder(ELL * 32);
        bytes memory buf = b.buf;

        Bytes.Builder memory builder = Bytes.newBuilder(H_IN_CHUNK_SIZE + message.length + 2 + 1 + DST_SUFFIX.length);
        builder.appendIntOSP(0, H_IN_CHUNK_SIZE);
        builder.appendBytes(message);
        builder.appendIntOSP(MSG_LEN, 2);
        builder.appendIntOSP(0, 1);
        builder.appendBytes(DST_SUFFIX);
        bytes32 b0 = sha256(builder.seal());

        builder = Bytes.newBuilder(32 + 1 + DST_SUFFIX.length);
        builder.appendBytes32(b0);
        builder.appendIntOSP(1, 1);
        builder.appendBytes(DST_SUFFIX);
        b.appendBytes32(sha256(builder.seal()));

        for (uint256 i = 2; i <= ELL; i++) {
            builder.reset();
            // append b[0] ^ b[i-1] 
            bytes32 xorVal;
            uint256 offset = b.offset;
            assembly {
                xorVal := xor(b0, mload(add(buf, offset)))
            }
            builder.appendBytes32(xorVal);
            builder.appendIntOSP(i, 1);
            builder.appendEmpty(DST_SUFFIX.length); // filled already
            b.appendBytes32(sha256(builder.seal()));
        }

        return b.seal();
    }

    function _inPlaceBigMod(Bytes.Builder memory builder, bytes memory buf, uint256 offset) private view {
        builder.reset();
        builder.appendEmpty(96);
        builder.appendBytes(buf, offset, L);
        builder.appendEmpty(1 + L);

        callPrecompile(PRECOMPILE_BIG_MOD_EXP, builder.seal(), buf, offset, L);
    }

    function _paddingAppend(Bytes.Builder memory builder, uint256 padding, bytes memory val, uint256 offset, uint256 len) private pure {
        builder.appendEmpty(padding);
        builder.appendBytes(val, offset, len);
    }

    function _appendPublicKey(Bytes.Builder memory builder, bytes memory publicKey) private pure {
        require(publicKey.length == 96 || publicKey.length == 128, "BLS: public key length mismatch");

        if (publicKey.length == 96) {
            _paddingAppend(builder, 16, publicKey, 0, 48);
            _paddingAppend(builder, 16, publicKey, 48, 48);
        } else {
            builder.appendBytes(publicKey);
        }
    }

    function _appendSignature(Bytes.Builder memory builder, bytes memory signature) private pure {
        require(signature.length == 192 || signature.length == 256, "BLS: signature length mismatch");

        if (signature.length == 192) {
            _paddingAppend(builder, 16, signature, 48, 48);
            _paddingAppend(builder, 16, signature, 0, 48);
            _paddingAppend(builder, 16, signature, 144, 48);
            _paddingAppend(builder, 16, signature, 96, 48);
        } else {
            builder.appendBytes(signature, 64, 64);
            builder.appendBytes(signature, 0, 64);
            builder.appendBytes(signature, 192, 64);
            builder.appendBytes(signature, 128, 64);
        }
    }

    function callPrecompile(address precompile, bytes memory input, bytes memory output) internal view {
        return callPrecompile(precompile, input, 0, input.length, output, 0, output.length);
    }

    function callPrecompile(address precompile, bytes memory input, bytes memory output, uint256 outputOffset, uint256 outputLen) internal view {
        return callPrecompile(precompile, input, 0, input.length, output, outputOffset, outputLen);
    }

    function callPrecompile(address precompile, 
        bytes memory input, uint256 inputOffset, uint256 inputLen,
        bytes memory output, uint256 outputOffset, uint256 outputLen
    ) internal view {
        require(inputOffset + inputLen <= input.length, "BLS: input out of bound");
        require(outputOffset + outputLen <= output.length, "BLS: output out of bound");

        bool success;

        assembly {
            let inputPtr := add(input, add(inputOffset, 32))
            let outputPtr := add(output, add(outputOffset, 32))
            success := staticcall(gas(), precompile, inputPtr, inputLen, outputPtr, outputLen)
        }

        require(success, string(abi.encodePacked("BLS: Failed to call pre-compile contract ", Strings.toHexString(precompile))));
    }

    // COMPRESSION_P = P / 2
    bytes32 private constant COMPRESSION_P_0 = 0x000000000000000000000000000000000d0088f51cbff34d258dd3db21a5d66b;
    bytes32 private constant COMPRESSION_P_1 = 0xb23ba5c279c2895fb39869507b587b120f55ffff58a9ffffdcff7fffffffd555;

    /**
     * @dev Compress public key into 48 bytes.
     */
    function compressPublicKey(bytes memory uncompressed) internal pure returns (bytes memory) {
        require(uncompressed.length == 96 || uncompressed.length == 128, "BLS: uncompressed public key length mismatch");

        bytes memory compressed = new bytes(48);
        bytes32 y0;
        bytes32 y1;

        if (uncompressed.length == 96) {
            assembly {
                y0 := mload(add(uncompressed, 80)) // header size (32 bytes) + x (48 bytes)
                y0 := shr(128, y0) // shift right 16 bytes
                y1 := mload(add(uncompressed, 96))
            }

            Bytes.memcopy(compressed, 0, uncompressed, 0, 48);
        } else {
            assembly {
                y0 := mload(add(uncompressed, 96)) // header size (32 bytes) + x (64 bytes)
                y1 := mload(add(uncompressed, 128))
            }

            Bytes.memcopy(compressed, 0, uncompressed, 16, 48);
        }

        compressed[0] |= 0x80; // compression flag
        if (y0 > COMPRESSION_P_0 || (y0 == COMPRESSION_P_0 && y1 > COMPRESSION_P_1)) {
            compressed[0] |= 0x20;
        }

        return compressed;
    }

}
