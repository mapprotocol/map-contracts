pragma solidity ^0.8.0;

import "./utils/CREATE3.sol";

contract DeployFactory {
    using Bytes32AddressLib for bytes32;

    event contractAddress(address indexed newaddr);

    bytes internal constant PROXY_BYTECODE = hex"67_36_3d_3d_37_36_3d_34_f0_3d_52_60_08_60_18_f3";

    bytes32 internal constant PROXY_BYTECODE_HASH = keccak256(PROXY_BYTECODE);

    function deployFactory(
        bytes32 salt,
        bytes memory creationCode,
        uint256 value) public {
        address newContract = CREATE3.deploy(salt,creationCode,value);

        emit contractAddress(newContract);
    }

    function getAddress(bytes32 salt) public view returns (address) {
        address proxy = keccak256(
            abi.encodePacked(
            // Prefix:
                bytes1(0xFF),
            // Creator:
                address(this),
            // Salt:
                salt,
            // Bytecode hash:
                PROXY_BYTECODE_HASH
            )
        ).fromLast20Bytes();

        return
        keccak256(
            abi.encodePacked(
            // 0xd6 = 0xc0 (short RLP prefix) + 0x16 (length of: 0x94 ++ proxy ++ 0x01)
            // 0x94 = 0x80 + 0x14 (0x14 = the length of an address, 20 bytes, in hex)
                hex"d6_94",
                proxy,
                hex"01" // Nonce of the proxy contract (1)
            )
        ).fromLast20Bytes();

    }

}
