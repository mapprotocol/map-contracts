// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./utils/Role.sol";
import "./interface/ILightClientManager.sol";
import "./interface/ILightNode.sol";


contract LightClientManager is ILightClientManager,Role {
    mapping(uint256 => address) lightClientContract;

    function register(uint256 _chainId, address _contract) external override onlyManager{
        lightClientContract[_chainId] = _contract;
    }

    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external override {
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        lightNode.updateBlockHeader(_blockHeader);
    }

    function verifyProofData(uint _chainId, bytes memory _receiptProof) external view override returns (bool success, bytes memory logs) {
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        return lightNode.verifyProofData(_receiptProof);
    }
}