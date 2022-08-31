// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "../lib/Verify.sol";

interface ILightNode {

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

    function updateBlockHeader(Verify.BlockHeader[] memory _blockHeaders) external;

    function headerHeight() external view returns (uint256);
}
