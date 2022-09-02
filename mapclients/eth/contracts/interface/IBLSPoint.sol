// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IBLSPoint {
    struct G1 {
        uint x;
        uint y;
    }
    struct G2 {
        uint xr;
        uint xi;
        uint yr;
        uint yi;
    }
}