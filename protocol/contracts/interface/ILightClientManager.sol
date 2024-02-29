// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightClientManager {
    event ManagerNotifySend(
        uint256 indexed chainId,
        address indexed sender,
        uint256 indexed blockHeight,
        bytes notifyData
    );

    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external;

    function updateLightClient(uint256 _chainId, bytes memory _data) external;

    function notifyLightClient(uint256 _chainId, address _from, bytes memory _data) external;

    function verifyProofDataWithCache(
        uint256 _chainId,
        bytes memory _receiptProof
    ) external returns (bool success, string memory message, bytes memory logs);

    function verifyProofData(
        uint256 _chainId,
        bytes memory _receiptProof
    ) external view returns (bool success, string memory message, bytes memory logs);

    function clientState(uint256 _chainId) external view returns (bytes memory);

    function headerHeight(uint256 _chainId) external view returns (uint256);

    function verifiableHeaderRange(uint256 _chainId) external view returns (uint256, uint256);

    function finalizedState(uint256 _chainId, bytes memory _data) external view returns (bytes memory);

    function isVerifiable(uint256 _chainId, uint256 _blockHeight, bytes32 _hash) external view returns (bool);

    function nodeType(uint256 _chainId) external view returns (uint256);
}
