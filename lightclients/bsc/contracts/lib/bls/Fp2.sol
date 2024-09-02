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

import "./Fp.sol";

struct Fp2 {
    Fp c0;
    Fp c1;
}

library FP2 {
    using FP for Fp;

    function eq(Fp2 memory x, Fp2 memory y) internal pure returns (bool) {
        return (x.c0.eq(y.c0) && x.c1.eq(y.c1));
    }

    function is_zero(Fp2 memory x) internal pure returns (bool) {
        return x.c0.is_zero() && x.c1.is_zero();
    }

    // Note: Zcash uses (x_im, x_re)
    function serialize(Fp2 memory x) internal pure returns (bytes memory) {
        return abi.encodePacked(x.c1.serialize(), x.c0.serialize());
    }
}