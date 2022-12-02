// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./VERC20.sol";
import "../interface/IVault.sol";
import "../utils/TransferHelper.sol";


contract MAPVaultToken is VERC20, IVault,Ownable {
    using SafeMath for uint;

    address public correspond;
    uint256 public correspondBalance;

    event VaultStaking(uint correspondAmount, uint vAmount);
    event VaultWithdraw(uint correspondAmount, uint vAmount);

    function initialize(
        address correspond_,
        string memory name_,
        string memory symbol_,
        uint8 decimals_) external {
        correspond = correspond_;
        init(name_, symbol_, decimals_);
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

    function stakingTo(uint amount, address to) external override onlyOwner {
        uint vToken = getVTokenQuantity(amount);
        _mint(to, vToken);
        correspondBalance += amount;
        emit VaultStaking(amount, vToken);
    }

    function addFee(uint amount) external override onlyOwner {
        correspondBalance += amount;
    }

    function withdraw(uint amount, address to) external override onlyOwner {
        uint correspondAmount = getCorrespondQuantity(amount);
        correspondBalance -= correspondAmount;
        _burn(to, amount);
        emit VaultWithdraw(correspondAmount, amount);
    }
}