# MAP Protocol management contracts

## Introduction


### Deploy Factory

The factory contract helps to deploy to deterministic addresses without an init code factor.
Every developer can use contract `0x6258e4d2950757A749a4d4683A7342261ce12471` to deploy deterministic addresses contract.

Read [this](./create3/README.md) to get more information.

Now support chains: 
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


### Light client manager
The contract deployed on the MAP Relay Chain is responsible for managing light clients, it helps:
- Register light client
- Verify cross chain proof
- Get light client verification range

### Maintainer manager
The contract deployed on the MAP Relay Chain is responsible for managing maintainers, it helps:
- staking
- reward distribution
- work address binding.


## Compile

Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd protocol
npm install
npx hardhat compile
```


## Test

```shell
npx hardhat test
```

## Deploy

```shell
npx hardhat deploy --tags MaintainerManager --network <network>

npx hardhat deploy --tags LightClientManager --network <network>
```

## Upgrade

```shell
npx hardhat deploy --tags MaintainerManagerUp --network <network>
```


## Useage

### Register a light client

cmd
```shell
npx hardhat clientRegister --chain <chain id for light client> --contract <contract for light client>  --network <network>
```

example
```shell
  npx hardhat LightClientRegister --chain 1 --contract "0x366db0D543b709434Cb91113270521e50fC2fe49" --network Map
```

### Add or remove a maintainer

cmd
```shell
 npx hardhat MaintainerWhileListSet --add <add:true remove:false> --address <Maintainer address> --network <network>
```
example

```shell
  npx hardhat MaintainerWhileListSet --add true --address "0x2f6950D5adE9025266677946c1E0233526387219" --network Map
```

## The contracts main

```shell
MaintainerManager 0x366db0D543b709434Cb91113270521e50fC2fe49
MaintainerManagerProxy 0x29B1bC29634e630967cf1A74973Bd0047880bE89
```