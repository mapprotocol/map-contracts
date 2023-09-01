// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

/// @notice DeployFactory address: 0x6258e4d2950757A749a4d4683A7342261ce12471
///          Support chains: https://github.com/mapprotocol/map-contracts/blob/main/protocol/contracts/create3/README.md
///
/// @author MAP Protocol (https://github.com/mapprotocol/map-contracts/blob/main/protocol/contracts/interface/IDeployFactory.sol)
/// @author Import CREATE3 library from Solmate (https://github.com/transmissions11/solmate/blob/main/src/utils/CREATE3.sol)
interface IDeployFactory {

    // @notice Deploy to deterministic addresses without an initcode factor.
    // @param salt - the bytes to deterministic address
    // @param creationCode - code to be deployed, include the init parameters.
    // @param value - native value when calling to deploy
    function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external;

    // @notice Get the deterministic addresses.
    // @param salt - the bytes to deterministic address
    function getAddress(bytes32 salt) external view returns (address);

}