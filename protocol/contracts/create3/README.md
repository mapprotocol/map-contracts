# Deploy Factory

The factory contract helps to deploy to deterministic addresses without an init code factor.


## Contract Address

Every developer can use contract `0x6258e4d2950757A749a4d4683A7342261ce12471` to deploy deterministic addresses contract.

## Contract Interface

```solidity
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
```

## Now support chains
- Ethereum (1), Goerli Testnet (5)
- BNB Smart Chain (56), BNB Smart Chain Testnet (97)
- Polygon (137), Mumbai Testnet (80001)
- MAP Relay Chain (22776), Makalu Testnet (212)
- Klaytn (8217), Klaytn Testnet (1001)
- Arbitrum (42161)
- Optimism (10)
- Avalanche (43114)
- Fantom (250)
- Gnosis Chain (100)
- Aurora (1313161554)
- Celo (42220)
- Harmony (1666600000)
- Polygon zkEVM (1101)
- Boba (288)
- Metis (1088)
- Linea (59144)
- Mantle (5000)
- Base (8453)
- Okt (66)
- Cronos (25)
- Kava EVM (2222)
- Evmos (9001)
- Filecoin EVM (314)
- Moonbeam (1284)
- Moonriver (1285)
- Astar (592)
- Conflux (1030)
- Oasis (42262)
- Velas (106)
- Telos (40)
- Syscoin (57)
- Fuse (122)
- Ethereum pow (10001)

## Will support
- Scroll (534352)

