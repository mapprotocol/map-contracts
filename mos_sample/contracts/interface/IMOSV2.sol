// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IMOSV2 {
    function transferOutToken(address _token, bytes memory _to, uint _amount, uint _toChain) external;

    event mapTransferOut(bytes token, bytes from, bytes32 orderId,
        uint256 fromChain, uint256 toChain, bytes to, uint256 amount, bytes toChainToken);

    event mapTransferIn(address indexed token, bytes from, bytes32 indexed orderId,
        uint256 fromChain, uint256 toChain, address to, uint256 amount);


}