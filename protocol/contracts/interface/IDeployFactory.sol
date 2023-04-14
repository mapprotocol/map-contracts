// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

/// @notice DeployFactory address: 0x6258e4d2950757A749a4d4683A7342261ce12471
///          Support chains: - Ethereum (1), Goerli Testnet (5)
///                          - BNB Smart Chain (56), BNB Smart Chain Testnet (97)
///                          - Polygon (137), Mumbai Testnet (80001)
///                          - MAP Relay Chain (22776), Makalu Testnet (212)
///                          - Klaytn (8217), Klaytn Testnet (1001)
///                          - Arbitrum (42161), Optimism (10)
///                          - Avalanche (43114), Fantom (250)
///                          - Gnosis Chain (100), Aurora (1313161554)
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