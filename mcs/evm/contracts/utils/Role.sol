// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/AccessControl.sol";

contract Role is AccessControl{
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    constructor(){
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(MANAGER_ROLE, msg.sender);
    }

    modifier onlyManager(){
        require(hasRole(MANAGER_ROLE, msg.sender), "Caller is not a manager");
        _;
    }

    function addManager(address manager) external onlyRole(DEFAULT_ADMIN_ROLE){
        _setupRole(MANAGER_ROLE, manager);
    }

    function removeManager(address manager) external onlyRole(DEFAULT_ADMIN_ROLE){
        _revokeRole(MANAGER_ROLE,manager);
    }
}