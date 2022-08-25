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
    uint accrualBlockNumber;
    mapping(address => uint) public userStakingAmount;

    address public correspond;
    IERC20 correspondToken;

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

    function correspondBalance() public view returns (uint){
        return IERC20(correspond).balanceOf(address(this));
    }

    function getVTokenQuantity(uint amount) public view returns (uint){
        if (totalSupply() == 0) {
            return amount;
        }
        uint allCorrespond = correspondBalance();
        uint allVToken = totalSupply();
        return amount.mul(allVToken).div(allCorrespond);
    }

    function getCorrespondQuantity(uint amount) public view returns (uint){
        uint allCorrespond = correspondBalance();
        uint allVToken = totalSupply();
        if (allVToken == 0) {
            return amount;
        }
        return amount.mul(allCorrespond).div(allVToken);
    }

    function staking(uint amount) external override {
        uint vtoken = getVTokenQuantity(amount);
        TransferHelper.safeTransferFrom(correspond, msg.sender, address(this), amount);
        _mint(msg.sender, vtoken);
        emit VaultStaking(amount, vtoken);
    }

    function stakingTo(uint amount, address to) external override onlyManager {
        uint vtoken = getVTokenQuantity(amount);
        _mint(to, vtoken);
        emit VaultStaking(amount, vtoken);
    }

    function withdraw(uint amount) external override {
        uint correspondAmount = getCorrespondQuantity(amount);
        require(correspondBalance().sub(correspondAmount) > 0, "take too much");
        _burn(msg.sender, amount);
        TransferHelper.safeTransferFrom(correspond, address(this),msg.sender, amount);
        emit VaultWithdraw(correspondAmount, amount);
    }
}