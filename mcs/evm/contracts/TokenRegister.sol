// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./interface/ITokenRegister.sol";


contract TokenRegister {
    uint public chainID;
    constructor(){
        uint _chainId;
        assembly {
            _chainId := chainid()
        }
        chainID = _chainId;
    }

    //Source chain to MAP chain
    mapping(uint256 => mapping(address => address)) public sourceCorrespond;
    //MAP chain to target
    mapping(uint256 => mapping(address => address)) public mapCorrespond;
    //Source token binding
    mapping(uint256 => mapping(address => address)) public sourceBinding;

    function regToken(
        uint256 sourceChain, address sourceMapToken, address mapToken
    ) external {
        sourceCorrespond[sourceChain][sourceMapToken] = mapToken;
        mapCorrespond[sourceChain][mapToken] = sourceMapToken;
    }

    function regTokenSource(address sourceToken, address sourceMapToken) external {
        sourceBinding[chainID][sourceMapToken] = sourceToken;
    }

    function getTargetToken(
        uint256 sourceChain, address sourceToken, uint256 targetChain
    ) external view  returns (address mapToken){
        return mapCorrespond[targetChain][sourceCorrespond[sourceChain][sourceToken]];
    }
}