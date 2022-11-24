// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface ITokenRegisterV2 {

    // Get token address on target chain
    function getToChainToken(address _token, uint256 _toChain) external view returns (bytes memory _toChainToken);

    // Get token amount on target chain
    function getToChainAmount(address _token, uint256 _amount, uint256 _toChain) external view returns (uint256);

    // Get token and vault token address on relay chain
    function getRelayChainToken(uint256 _fromChain, bytes memory _fromToken) external view returns (address);

    // Get token amount on relay chain
    function getRelayChainAmount(address _token, uint256 _fromChain, uint256 _amount) external view returns (uint256);

    function checkMintable(address _token) external view returns (bool);

    function getVaultToken(address _token) external view returns (address);

    function getTokenFee(address _token, uint256 _amount, uint256 _toChain) external view returns (uint256);
}