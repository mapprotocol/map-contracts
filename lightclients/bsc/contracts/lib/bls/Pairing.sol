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

import "./G1.sol";
import "./G2.sol";

library Pairing {
    uint8 private constant PAIRING = 0x10;

    function negativeP1() internal pure returns (G1Point memory p) {
        p.x.a = 31827880280837800241567138048534752271;
        p.x.b = 88385725958748408079899006800036250932223001591707578097800747617502997169851;
        p.y.a = 22997279242622214937712647648895181298;
        p.y.b = 46816884707101390882112958134453447585552332943769894357249934112654335001290;
    }

    // e(P,Q) * e(R,S)
    function pairing(G1Point memory p, G2Point memory q, G1Point memory r, G2Point memory s) internal view returns (bool) {
        uint[24] memory input;
        input[0]  = p.x.a;
        input[1]  = p.x.b;
        input[2]  = p.y.a;
        input[3]  = p.y.b;
        input[4]  = q.x.c0.a;
        input[5]  = q.x.c0.b;
        input[6]  = q.x.c1.a;
        input[7]  = q.x.c1.b;
        input[8]  = q.y.c0.a;
        input[9]  = q.y.c0.b;
        input[10] = q.y.c1.a;
        input[11] = q.y.c1.b;
        input[12] = r.x.a;
        input[13] = r.x.b;
        input[14] = r.y.a;
        input[15] = r.y.b;
        input[16] = s.x.c0.a;
        input[17] = s.x.c0.b;
        input[18] = s.x.c1.a;
        input[19] = s.x.c1.b;
        input[20] = s.y.c0.a;
        input[21] = s.y.c0.b;
        input[22] = s.y.c1.a;
        input[23] = s.y.c1.b;
        uint[1] memory output;

        assembly ("memory-safe") {
            if iszero(staticcall(161000, PAIRING, input, 768, output, 32)) {
                let pt := mload(0x40)
                returndatacopy(pt, 0, returndatasize())
                revert(pt, returndatasize())
            }
        }

        return output[0] == 1;
    }
}