// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IBLSPoint {
    struct G1 {
        uint256 x;
        uint256 y;
    }
    struct G2 {
        uint256 xr;
        uint256 xi;
        uint256 yr;
        uint256 yi;
    }
}
