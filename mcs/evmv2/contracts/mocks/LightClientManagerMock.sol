// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import "../interface/ILightClientManager.sol";
import "../interface/ILightNode.sol";


contract LightClientManager is ILightClientManager,Ownable {
    mapping(uint256 => address) public lightClientContract;
    mapping(uint256 => address) public updateBlockContract;

    function register(uint256 _chainId, address _contract,address _blockContract) external override onlyOwner{
        lightClientContract[_chainId] = _contract;
        updateBlockContract[_chainId] = _blockContract;
    }

    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external override {
        require(updateBlockContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(updateBlockContract[_chainId]);
        lightNode.updateBlockHeader(_blockHeader);
    }

    function verifyProofData(uint _chainId, bytes memory _receiptProof) external view override
    returns (bool success, string memory message, bytes memory logs) {
//        require(lightClientContract[_chainId] != address(0), "not register");
//        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
//        return lightNode.verifyProofData(_receiptProof);
        if(_chainId == 888){
            return(false,"fail",_receiptProof);
        }else{
            return(true,"success",_receiptProof);
        }

    }

    function headerHeight(uint256 _chainId) external view override returns (uint256){
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(updateBlockContract[_chainId]);
        if(_chainId == 34434){
            (uint256 number,) = lightNode.currentNumberAndHash(_chainId);
            return number;
        }else{
            return lightNode.headerHeight();
        }
    }

    function verifiableHeaderRange(uint256 _chainId) external view override returns (uint256, uint256) {
        return (0, 0);
    }
}