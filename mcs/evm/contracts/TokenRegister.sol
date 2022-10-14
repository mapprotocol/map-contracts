// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./interface/ITokenRegister.sol";

contract TokenRegister is Ownable {
    uint public immutable chainID = block.chainid;

    constructor(){
    }

    //Source chain to MAP chain
    mapping(uint256 => mapping(bytes => bytes)) public sourceCorrespond;
    //MAP chain to target
    mapping(uint256 => mapping(bytes => bytes)) public mapCorrespond;
    //Source token binding
    mapping(uint256 => mapping(bytes => bytes)) public sourceBinding;

    function regToken(
        uint256 sourceChain, bytes memory sourceMapToken, bytes memory mapToken
    ) external
    onlyOwner{
        sourceCorrespond[sourceChain][sourceMapToken] = mapToken;
        mapCorrespond[sourceChain][mapToken] = sourceMapToken;
    }

    function regTokenSource(bytes memory sourceToken, bytes memory sourceMapToken) external
    onlyOwner{
        sourceBinding[chainID][sourceMapToken] = sourceToken;
    }

    function getTargetToken(
        uint256 sourceChain, bytes memory sourceToken, uint256 targetChain
    ) external view  returns (bytes memory mapToken){
        if(targetChain == chainID ){
            mapToken = sourceCorrespond[sourceChain][sourceToken];
        }else if(sourceChain == chainID){
            mapToken = mapCorrespond[targetChain][sourceToken];
        }else{
            mapToken = mapCorrespond[targetChain][sourceCorrespond[sourceChain][sourceToken]];
        }

    }
}