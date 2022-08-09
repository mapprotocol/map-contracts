// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ITokenRegister {
    function sourceCorrespond(uint256 sourceChainID, bytes memory sourceMapToken) external view returns (bytes memory mapToken);

    function mapCorrespond(uint256 sourceChainID, bytes memory sourceMapToken) external view returns (bytes memory sourceToken);

    function sourceBinding(uint256 sourceChainID, bytes memory sourceMapToken) external view returns (bytes memory mapToken);

    function getTargetToken(uint256 sourceChain, bytes memory sourceToken, uint256 targetChain) external view returns (bytes memory mapToken);

    function regToken(uint256 sourceChain, bytes memory sourceMapToken, bytes memory mapToken) external;

    function regTokenSource(bytes memory sourceToken, bytes memory sourceMapToken) external;
}