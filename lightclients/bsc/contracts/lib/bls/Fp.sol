// This file is part of Darwinia.
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

pragma solidity 0.8.17;

import "./Bytes.sol";

struct Fp {
    uint a;
    uint b;
}

library FP {
    using Bytes for bytes;
    uint8 private constant BIG_MOD_EXP = 0x05;

    // Base field modulus = 0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab
    function q() internal pure returns (Fp memory) {
        return Fp(0x1a0111ea397fe69a4b1ba7b6434bacd7, 0x64774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab);
    }

    function zero() internal pure returns (Fp memory) {
        return Fp(0, 0);
    }

    function is_valid(Fp memory x) internal pure returns (bool) {
        return gt(q(), x);
    }

    function is_zero(Fp memory x) internal pure returns (bool) {
        return eq(x, zero());
    }

    function eq(Fp memory x, Fp memory y) internal pure returns (bool) {
        return (x.a == y.a && x.b == y.b);
    }

    function gt(Fp memory x, Fp memory y) internal pure returns (bool) {
        return (x.a > y.a || (x.a == y.a && x.b > y.b));
    }

    function add(Fp memory x, Fp memory y) internal pure returns (Fp memory z) {
        unchecked {
            z.b = x.b + y.b;
            z.a = x.a + y.a + (z.b >= x.b && x.b >= y.b ? 0 : 1);
        }
    }

    function serialize(Fp memory x) internal pure returns (bytes memory) {
        return abi.encodePacked(uint128(x.a), x.b);
    }

    // function deserialize(bytes memory xf) internal pure returns (Fp memory x) {
    //     x = Fp(xf.slice_to_uint(0, 16), xf.slice_to_uint(16, 48));
    // }

    function norm(Fp memory base) internal view returns (Fp memory) {
        uint[8] memory input;
        input[0] = 0x40;
        input[1] = 0x20;
        input[2] = 0x40;
        input[3] = base.a;
        input[4] = base.b;
        input[5] = 1;
        input[6] = q().a;
        input[7] = q().b;
        uint[2] memory output;

        assembly ("memory-safe") {
            if iszero(staticcall(sub(gas(), 2000), BIG_MOD_EXP, input, 256, output, 64)) {
                let p := mload(0x40)
                returndatacopy(p, 0, returndatasize())
                revert(p, returndatasize())
            }
        }
        return Fp(output[0], output[1]);
    }
}