// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";


contract LightNodeProxy is ERC1967Proxy {

    //0x439fab91000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000066175726f72610000000000000000000000000000000000000000000000000000
    constructor(address _logic, bytes memory _data)
        payable
        ERC1967Proxy(_logic, _data)
    {
         require(address(_logic) != address(0),"_logic zero address");
    }
}