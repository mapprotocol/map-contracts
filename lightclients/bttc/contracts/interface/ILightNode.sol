// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface ILightNode {

    event UpdateBlockHeader(address indexed account, uint256 indexed blockHeight);

    //Verify the validity of the transaction according to the header, receipt
    //The interface will be updated later to return logs
    function verifyProofData(bytes memory _receiptProof)
        external
        view
        returns (
            bool success,
            string memory message,
            bytes memory logsHash
        );

    //Validate headers and update validation members

    function updateBlockHeader(bytes memory _blockHeaders) external;

    function headerHeight() external view returns (uint256);

    function verifiableHeaderRange() external view returns (uint256, uint256);
}
