// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./interface/ITokenRegister.sol";

contract TokenRegister is Ownable, ITokenRegister {
    using SafeMath for uint;
    uint public immutable chainID = block.chainid;

    //chainId token decimals
    mapping(uint256 => mapping(bytes => uint256)) tokenOtherChainDecimals;

    //Source chain to MAP chain
    mapping(uint256 => mapping(bytes => bytes)) public sourceCorrespond;
    //MAP chain to target
    mapping(uint256 => mapping(bytes => bytes)) public mapCorrespond;

    function regToken(uint256 sourceChain, bytes memory sourceMapToken, bytes memory mapToken)
    external
    onlyOwner{
        sourceCorrespond[sourceChain][sourceMapToken] = mapToken;
        mapCorrespond[sourceChain][mapToken] = sourceMapToken;
    }


    function getTargetToken(uint256 sourceChain, bytes memory sourceToken, uint256 targetChain)
    external override
    view
    returns (bytes memory mapToken){
        if(targetChain == chainID ){
            mapToken = sourceCorrespond[sourceChain][sourceToken];
        }else if(sourceChain == chainID){
            mapToken = mapCorrespond[targetChain][sourceToken];
        }else{
            mapToken = mapCorrespond[targetChain][sourceCorrespond[sourceChain][sourceToken]];
        }
    }

    function setTokenOtherChainDecimals(bytes memory selfToken, uint256 chainId, uint256 decimals)
    external
    onlyOwner {
        tokenOtherChainDecimals[chainId][selfToken] = decimals;
    }


    function getToChainAmount(bytes memory token, uint256 fromChain, uint256 toChain, uint256 amount)
    external override
    view
    returns (uint256){
        uint256 decimalsFrom = tokenOtherChainDecimals[fromChain][token];
        uint256 decimalsTo = tokenOtherChainDecimals[toChain][token];
        return amount.mul(10 ** decimalsTo).div(10 ** decimalsFrom);
    }
}