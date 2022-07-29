// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ILightNode {
    //Verify the validity of the transaction according to the header, receipt, and aggPk
    //The interface will be updated later to return logs
    function verifyProofData(bytes memory _receiptProof)
        external
        returns (
            bool success,
            string memory message,
            bytes memory logsHash
        );

    //Validate headers and update validation members

    function updateBlockHeader(bytes memory _blockHeader) external;

    function headerHeight() external view returns (uint256);
}
