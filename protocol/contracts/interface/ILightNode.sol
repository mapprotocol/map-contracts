// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import {ILightVerifier} from "./ILightVerifier.sol";

interface ILightNode is ILightVerifier {

    event UpdateBlockHeader(address indexed maintainer, uint256 indexed blockHeight);

    function updateBlockHeader(bytes memory _blockHeader) external;

    function updateLightClient(bytes memory _data) external;

    function finalizedState(bytes memory _data) external view returns (bytes memory);

    // Get client state
    function clientState() external view returns (bytes memory);

    // @notice Get the light client block height
    // @return height - current block height or slot number
    function headerHeight() external view returns (uint256 height);

}
