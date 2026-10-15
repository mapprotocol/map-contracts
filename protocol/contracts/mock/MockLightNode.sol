// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "../interface/ILightNode.sol";

/// @dev Test double of a light client. Records the calls it receives from the manager.
contract MockLightNode is ILightNode {
    bool public fail;
    uint256 public override nodeType = 1;
    uint256 public override headerHeight = 100;
    uint256 public minRange = 10;
    uint256 public maxRange = 100;
    bytes public state = hex"c0ffee";

    address public lastCaller;
    bytes public lastBlockHeader;
    bytes public lastUpdateData;
    uint256 public cacheCalls;

    function setFail(bool _fail) external {
        fail = _fail;
    }

    function setNodeType(uint256 _type) external {
        nodeType = _type;
    }

    function updateBlockHeader(bytes memory _blockHeader) external override {
        require(!fail, "mock: failed");
        lastCaller = msg.sender;
        lastBlockHeader = _blockHeader;
        emit UpdateBlockHeader(msg.sender, headerHeight);
    }

    function updateLightClient(bytes memory _data) external override {
        require(!fail, "mock: failed");
        lastCaller = msg.sender;
        lastUpdateData = _data;
    }

    function finalizedState(bytes memory _data) external pure override returns (bytes memory) {
        return abi.encodePacked("finalized:", _data);
    }

    function clientState() external view override returns (bytes memory) {
        return state;
    }

    function notifyLightClient(address, bytes memory) external pure override {}

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external override returns (bool success, string memory message, bytes memory logs) {
        require(!fail, "mock: failed");
        cacheCalls++;
        return (true, "", _receiptProof);
    }

    function verifyProofDataWithCache(
        bool,
        uint256,
        bytes memory
    ) external pure override returns (bool, string memory, txLog memory log) {
        return (true, "", log);
    }

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        require(!fail, "mock: failed");
        return (true, "", _receiptProof);
    }

    function verifyProofData(
        uint256,
        bytes memory
    ) external pure override returns (bool, string memory, txLog memory log) {
        return (true, "", log);
    }

    function verifiableHeaderRange() external view override returns (uint256, uint256) {
        return (minRange, maxRange);
    }

    function isVerifiable(uint256 _blockHeight, bytes32) external view override returns (bool) {
        return _blockHeight >= minRange && _blockHeight <= maxRange;
    }
}
