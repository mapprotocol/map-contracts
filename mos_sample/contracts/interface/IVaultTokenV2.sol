// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IVaultTokenV2 {

    function deposit(uint256 _fromChain, uint256 _amount, address _to) external;

    function withdraw(uint256 _toChain, uint256 _vaultAmount, address _to) external;

    function transferToken(uint256 _fromChain, uint256 _amount,  uint256 _toChain, uint256 _outAmount, uint256 _relayChain, uint256 _fee) external;

    function getTokenAmount(uint256 _amount) external view returns (uint256);

    function getTokenAddress() external view returns (address);
}