// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface ITokenRegister {
    function sourceCorrespond(uint256 sourceChainID, address sourceMapToken) external view returns (address mapToken);

    function mapCorrespond(uint256 sourceChainID, address sourceMapToken) external view returns (address sourceToken);

    function sourceBinding(uint256 sourceChainID, address sourceMapToken) external view returns (address mapToken);

    function getTargetToken(uint256 sourceChain, address sourceToken, uint256 targetChain) external view returns (address mapToken);

    function regToken(uint256 sourceChain, address sourceMapToken, address mapToken) external;

    function regTokenSource(address sourceToken, address sourceMapToken) external;
}