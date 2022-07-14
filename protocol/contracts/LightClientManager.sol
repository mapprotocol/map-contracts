// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./utils/Role.sol";
import "./interface/ILightClientManager.sol";
import "./interface/ILightNode.sol";


contract LightClientManager is ILightClientManager {
    mapping (uint256 => address) lightClientContract;
    function updateBlockHeader(uint256 _chainId, bytes memory _blackHeader) external{
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        lightNode.updateBlockHeader(_blackHeader);
    }
    function register(uint256 _chainId, address _contract) external{
        lightClientContract[_chainId] = _contract;
    }
}