// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/token/ERC20/presets/ERC20PresetMinterPauser.sol";

contract MintableToken is ERC20PresetMinterPauser {
    uint8 private _decimals;

    constructor(string memory name, string memory symbol, uint8 decimals) ERC20PresetMinterPauser(name, symbol) {
        _decimals = decimals;
    }

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }
}