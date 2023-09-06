// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";


contract RootERC20 is ERC20 {
    constructor(string memory name_, string memory symbol_) ERC20(name_, symbol_)
    {
        uint256 amount = 10**10 * (10**18);
        _mint(msg.sender, amount);
    }

}

//test rootManager TV714HHcDKhrCrvufcdBz68frRvtbLgXa5
//rootManager TV714HHcDKhrCrvufcdBz68frRvtbLgXa5
//child manager 0xfe22C61F33e6d39c04dE80B7DE4B1d83f75210C4
//root erc20 TKTsTJuVajETtyczNvUgwizPSRfPXQvfst
//26239743
//26239679
//64

//mainet
// rootToken TLG9L7PYXuHY8cot4pZc6QFpcXdGYWhYvC
//child token 0x60093ee49Ec47E5d5D53cd77889bD4E997E6900e