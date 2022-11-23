// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract TokenRegisterProxy is ERC1967Proxy {

    constructor(address _logic, bytes memory _data)
    payable
    ERC1967Proxy(_logic, _data)
    {
        require(address(_logic) != address(0), "_logic zero address");
    }

}
