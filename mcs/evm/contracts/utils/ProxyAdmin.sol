// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;


import "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

contract ProxyAdminImport is ProxyAdmin{
    function getInitCallData(address wCoin, address map) public pure returns (bytes memory){
        bytes4 fun = bytes4(keccak256(bytes('initialize(address,address)')));
        return abi.encodeWithSelector(fun,wCoin,map);
    }
}
