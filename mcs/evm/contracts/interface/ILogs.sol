// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILogs {
    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }
}