// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./VERC20.sol";
import "../interface/IVaultTokenV2.sol";
import "../utils/Role.sol";
import "../utils/TransferHelper.sol";


contract MAPVaultToken is VERC20, IVaultTokenV2, Role {
    using SafeMath for uint256;

    // chain_id => vault_value
    mapping(uint256 => int256) public vaultBalance;

    address public correspondToken;
    uint256 public totalVault;

    event DepositVault(address indexed token, address indexed to, uint256 vaultValue, uint256 value);
    event WithdrawVault(address indexed token, address indexed to, uint256 vaultValue, uint256 value);

    function initialize(
        address correspond_,
        string memory name_,
        string memory symbol_,
        uint8 decimals_) external {
        correspondToken = correspond_;
        init(name_, symbol_, decimals_);
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(MANAGER_ROLE, msg.sender);
    }

    function getVaultTokenAmount(uint256 _amount) public view returns (uint){
        if (totalSupply() == 0) {
            return _amount;
        }
        uint allVToken = totalSupply();
        require(totalVault > 0, "getVTokenQuantity/correspondBalance is zero");
        return _amount.mul(allVToken).div(totalVault);
    }

    function getTokenAmount(uint256 _amount) public override view returns (uint256){
        uint allVToken = totalSupply();
        if (allVToken == 0) {
            return _amount;
        }
        return _amount.mul(totalVault).div(allVToken);
    }

    function getTokenAddress() public override view returns (address){
        return correspondToken;
    }

    function deposit(uint256 _fromChain, uint256 _amount, address _to) external override onlyManager {
        uint256 amount = getVaultTokenAmount(_amount);
        _mint(_to, amount);
        //_setVaultValue(_fromChain, _amount, 0, 0);

        vaultBalance[_fromChain] += int256(amount);
        totalVault += amount;

        emit DepositVault(correspondToken, _to, _amount, amount);
    }

    function withdraw(uint256 _toChain, uint256 _amount, address _to) external override onlyManager {
        uint amount = getTokenAmount(_amount);
        _burn(_to, _amount);
        //_setVaultValue(0, 0, _toChain, amount);

        vaultBalance[_toChain] -= int256(amount);
        totalVault -= amount;

        emit WithdrawVault(correspondToken, _to, _amount, amount);
    }

    function transferToken(uint256 _fromChain, uint256 _amount,  uint256 _toChain, uint256 _outAmount, uint256 _relayChain, uint256 _fee) external override onlyManager {
        vaultBalance[_fromChain] += int256(_amount);
        vaultBalance[_toChain] -= int256(_outAmount);
        vaultBalance[_relayChain] -= int256(_fee);
        totalVault = totalVault + _amount - _outAmount - _fee;
    }
}