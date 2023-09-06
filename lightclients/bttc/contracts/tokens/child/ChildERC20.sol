// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {AccessControlMixin} from "./abstract/AccessControlMixin.sol";
import {IChildToken} from "./interfaces/IChildToken.sol";
import {NativeMetaTransaction} from "./abstract/NativeMetaTransaction.sol";
import {ContextMixin} from "./abstract/ContextMixin.sol";


contract ChildERC20 is
    ERC20,
    IChildToken,
    AccessControlMixin,
    NativeMetaTransaction,
    ContextMixin
{

    mapping (bytes32 => uint256) public deposits;
    bytes32 public constant DEPOSITOR_ROLE = keccak256("DEPOSITOR_ROLE");

    event  ReceiveFormRootChain(address user,bytes depositData);

    constructor(
        string memory name_,
        string memory symbol_,
        address childChainManager
    )  ERC20(name_, symbol_) {
        _setupContractId("ChildERC20");
        _setupRole(DEFAULT_ADMIN_ROLE, _msgSender());
        _setupRole(DEPOSITOR_ROLE, childChainManager);
        _initializeEIP712(name_);
    }

    // This is to support Native meta transactions
    // never use msg.sender directly, use _msgSender() instead
    function _msgSender()internal override view returns (address sender)
    {
        return ContextMixin.msgSender();
    }

    /**
     * @notice called when token is deposited on root chain
     * @dev Should be callable only by ChildChainManager
     * Should handle deposit by minting the required amount for user
     * Make sure minting is done only by this function
     * @param user user address for whom deposit is being done
     * @param depositData abi encoded amount
     */
    function deposit(address user, bytes calldata depositData)
        external
        override
        only(DEPOSITOR_ROLE)
    {
        uint256 amount = abi.decode(depositData, (uint256));
        _mint(user, amount);
        deposits[keccak256(depositData)] = block.timestamp;
        emit ReceiveFormRootChain(user,depositData);
    }

    /**
     * @notice called when user wants to withdraw tokens back to root chain
     * @dev Should burn user's tokens. This transaction will be verified when exiting on root chain
     * @param amount amount of tokens to withdraw
     */
    function withdrawTo(address to, uint256 amount) public {
        _burn(_msgSender(), amount);
        emit WithdrawTo(to, address(0x00), amount);
    }

    function withdraw(uint256 amount) external {
        withdrawTo(_msgSender(), amount);
    }
}