// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

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

    address public underlying;
    // chain_id => vault_value
    mapping(uint256 => int256) public vaultBalance;
    uint256 public totalVault;

    uint8 private _decimals;

    event DepositVault(address indexed token, address indexed to, uint256 vaultValue, uint256 value);
    event WithdrawVault(address indexed token, address indexed to, uint256 vaultValue, uint256 value);


    /**
     * @dev Grants `DEFAULT_ADMIN_ROLE`, `MANAGER_ROLE` to the
     * account that deploys the contract.
     *
     * See {ERC20-constructor}.
     */
    constructor(address _underlying, string memory _name, string memory _symbol) ERC20(_name, _symbol) {
        require(_underlying != address(0), "underlying address is zero");
        _setupRole(DEFAULT_ADMIN_ROLE, _msgSender());

        _setupRole(MANAGER_ROLE, _msgSender());

        underlying = _underlying;

        _decimals = IERC20Metadata(underlying).decimals();
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

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }

    function getVaultTokenAmount(uint256 _amount) public view returns (uint256){
        if (totalSupply() == 0) {
            return _amount;
        }
        uint allVToken = totalSupply();
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

        vaultBalance[_fromChain] += int256(_amount);
        totalVault += _amount;

        emit DepositVault(underlying, _to, _amount, amount);
    }

    function withdraw(uint256 _toChain, uint256 _vaultAmount, address _to) external override onlyManager {
        uint256 amount = getTokenAmount(_vaultAmount);
        _burn(_to, _vaultAmount);

        vaultBalance[_toChain] -= int256(amount);
        totalVault -= amount;

        emit WithdrawVault(underlying, _to, _vaultAmount, amount);
    }

    function transferToken(uint256 _fromChain, uint256 _amount,  uint256 _toChain, uint256 _outAmount, uint256 _relayChain, uint256 _fee) external override onlyManager {
        vaultBalance[_fromChain] += int256(_amount);
        vaultBalance[_toChain] -= int256(_outAmount);

        uint256 fee = _amount - _outAmount - _fee;
        vaultBalance[_relayChain] += int256(fee);
        totalVault += fee;
    }
}