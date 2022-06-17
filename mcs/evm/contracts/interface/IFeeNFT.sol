// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IFeeNFT {
    function getChainNFTFee(uint fromChain, uint toChain) external view returns(uint);
    function getToChainNFTFee(uint toChain) external view returns(uint);
    function getChainNativeToken(uint chain) external view  returns(address);
    function getChainNativeTokenAndFee(uint fromChain,uint toChain) external view  returns(address token ,uint fee);
}