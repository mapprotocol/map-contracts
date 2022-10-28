// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ITokenRegister {
    function getTargetToken(uint256 sourceChain, bytes memory sourceToken, uint256 targetChain) external view returns (bytes memory mapToken);

    function getToChainAmount(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount) external view returns (uint256);
}