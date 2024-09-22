// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@mapprotocol/protocol/contracts/interface/ILightVerifier.sol";

interface ILightNode is ILightVerifier {

    event UpdateBlockHeader(address indexed account, uint256 indexed blockHeight);

}
