// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/AccessControlEnumerable.sol";
import "../interface/IVaultTokenV2.sol";
import "../utils/TransferHelper.sol";


contract VaultTokenV2 is IVaultTokenV2, AccessControlEnumerable,ERC20Burnable {
    using SafeMath for uint256;

    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    // chain_id => vault_value
    mapping(uint256 => int256) public vaultBalance;

    address public underlying;
    uint256 public totalVault;

    event DepositVault(address indexed token, address indexed to, uint256 vaultValue, uint256 value);
    event WithdrawVault(address indexed token, address indexed to, uint256 vaultValue, uint256 value);


    /**
     * @dev Grants `DEFAULT_ADMIN_ROLE`, `MANAGER_ROLE` to the
     * account that deploys the contract.
     *
     * See {ERC20-constructor}.
     */
    constructor(address _underlying, string memory _name, string memory _symbol) ERC20(_name, _symbol) {
        _setupRole(DEFAULT_ADMIN_ROLE, _msgSender());

        _setupRole(MANAGER_ROLE, _msgSender());

        underlying = _underlying;
    }

    modifier onlyManager(){
        require(hasRole(MANAGER_ROLE, msg.sender), "Caller is not a manager");
        _;
    }

    function addManager(address _manager) external onlyRole(DEFAULT_ADMIN_ROLE){
        _setupRole(MANAGER_ROLE, _manager);
    }

    function removeManager(address _manager) external onlyRole(DEFAULT_ADMIN_ROLE){
        _revokeRole(MANAGER_ROLE, _manager);
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
        return underlying;
    }

    function deposit(uint256 _fromChain, uint256 _amount, address _to) external override onlyManager {
        uint256 amount = getVaultTokenAmount(_amount);
        _mint(_to, amount);
        //_setVaultValue(_fromChain, _amount, 0, 0);

        vaultBalance[_fromChain] += int256(amount);
        totalVault += amount;

        emit DepositVault(underlying, _to, _amount, amount);
    }

    function withdraw(uint256 _toChain, uint256 _amount, address _to) external override onlyManager {
        uint amount = getTokenAmount(_amount);
        _burn(_to, _amount);
        //_setVaultValue(0, 0, _toChain, amount);

        vaultBalance[_toChain] -= int256(amount);
        totalVault -= amount;

        emit WithdrawVault(underlying, _to, _amount, amount);
    }

    function transferToken(uint256 _fromChain, uint256 _amount,  uint256 _toChain, uint256 _outAmount, uint256 _relayChain, uint256 _fee) external override onlyManager {
        vaultBalance[_fromChain] += int256(_amount);
        vaultBalance[_toChain] -= int256(_outAmount);
        vaultBalance[_relayChain] -= int256(_fee);
        totalVault = totalVault + _amount - _outAmount - _fee;
    }
}