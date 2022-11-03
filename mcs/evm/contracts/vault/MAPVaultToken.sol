// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./VERC20.sol";
import "../interface/IVault.sol";
import "../utils/Role.sol";
import "../utils/TransferHelper.sol";


contract MAPVaultToken is VERC20, IVault, Role {
    using SafeMath for uint;
    mapping(address => uint) public userStakingAmount;

    address public correspond;
    IERC20 public correspondToken;
    uint256 public correspondBalance;

    event VaultStaking(uint correspondAmount, uint vAmount);
    event VaultWithdraw(uint correspondAmount, uint vAmount);

    function initialize(
        address correspond_,
        string memory name_,
        string memory symbol_,
        uint8 decimals_) external {
        correspond = correspond_;
        correspondToken = IERC20(correspond);
        init(name_, symbol_, decimals_);
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(MANAGER_ROLE, msg.sender);
    }

    function getVTokenQuantity(uint amount) public view returns (uint){
        if (totalSupply() == 0) {
            return amount;
        }
        uint allVToken = totalSupply();
        require(correspondBalance > 0, "getVTokenQuantity/correspondBalance is zero");
        return amount.mul(allVToken).div(correspondBalance);
    }

    function getCorrespondQuantity(uint amount) public override view returns (uint){
        uint allVToken = totalSupply();
        if (allVToken == 0) {
            return amount;
        }
        return amount.mul(correspondBalance).div(allVToken);
    }

    function stakingTo(uint amount, address to) external override onlyManager {
        uint vToken = getVTokenQuantity(amount);
        _mint(to, vToken);
        correspondBalance += amount;
        emit VaultStaking(amount, vToken);
    }

    function addFee(uint amount) external override onlyManager {
        correspondBalance += amount;
    }

    function withdraw(uint amount, address to) external override onlyManager {
        _burn(to, amount);
        correspondBalance -= amount;
        emit VaultWithdraw(getCorrespondQuantity(amount), amount);
    }
}