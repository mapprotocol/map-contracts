// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./IVerifyTool.sol";

interface ILightNode {
    event UpdateBlockHeader(address indexed account, uint256 indexed blockHeight);

    event ClientNotifySend(address indexed sender, uint256 indexed blockHeight, bytes notifyData);

    function verifyProofDataWithCache(
        bytes memory _receiptProofBytes
    ) external returns (bool success, string memory message, bytes memory logs);

    // @notice Notify light client to relay the block
    // @param _data - notify data, if no data set it to empty
    function notifyLightClient(address _from, bytes memory _data) external;

    //Validate headers and update validation members
    //function updateBlockHeader(blockHeader memory bh, istanbulExtra memory ist, G2 memory aggPk) external;

    //Verify the validity of the transaction according to the header, receipt, and aggPk
    //The interface will be updated later to return logs
    function verifyProofData(
        bytes memory _receiptProof
    ) external returns (bool success, string memory message, bytes memory logsHash);

    function verifiableHeaderRange() external view returns (uint256, uint256);

    // @notice Check whether the block can be verified
    // @return
    function isVerifiable(uint256 _blockHeight, bytes32 _hash) external view returns (bool);

    // @notice Get the light client type
    // @return - 1 default light client
    //           2 zk light client
    //           3 oracle client
    function nodeType() external view returns (uint256);
}
