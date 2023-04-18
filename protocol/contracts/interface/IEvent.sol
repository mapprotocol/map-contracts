// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IEvent {

    struct txLog {
        address addr;
        bytes[] topics;
        bytes data;
    }
}