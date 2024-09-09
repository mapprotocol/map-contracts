# MAP Protocol management contracts

## Introduction


### Deploy Factory

The factory contract helps to deploy to deterministic addresses without an init code factor.
Every developer can use contract `0x6258e4d2950757A749a4d4683A7342261ce12471` to deploy deterministic addresses contract.

Read [this](https://github.com/mapprotocol/map-contracts/blob/main/protocol/contracts/create3/README.md) to get more information.

### MPT verifier
The contract deployed on the MAP Relay Chain is responsible for MPT proof verify.
Deployed address: 
 - MAPO mainnet: `0x81D26E2387059CF43ADA1c11c12D5d6627184fA1`
 - Makalu testnet: `0x81D26E2387059CF43ADA1c11c12D5d6627184fA1`

```solidity
interface IMPTVerify {
    function verifyTrieProof(
        bytes32 _root,
        bytes memory _key,
        bytes[] memory _proof,
        bytes memory _node
    ) external pure returns (bool);
}
```

### Light client manager
The contract deployed on the MAP Relay Chain is responsible for managing light clients, it helps:
- Register light client
- Verify cross chain proof
- Get light client verification range

Deployed address:
- MAPO mainnet: `0x624E6F327c4F91F1Fa6285711245c215de264d49`
- Makalu testnet: `0xDD3A69f8f59d892476B0be0260932b4f8d8268Ff`

### Maintainer manager
The contract deployed on the MAP Relay Chain is responsible for managing maintainers, it helps:
- staking
- reward distribution
- work address binding.

Deployed address:
- MAPO mainnet: `0x2ef591cAac00d21F16d25988678856244205c56E`
- Makalu testnet: `0xd0A42e4836A8827cB9b8A3e13f07abdC5610aC33`

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

npx hardhat deploy --tags LightClientManagerUp --network <network>
```


## Useage

### Register a light client

cmd
```shell
npx hardhat clientRegister --chain <chain id for light client> --contract <contract for light client>  --network <network>
```

example
```shell
  npx hardhat clientRegister --chain 1 --contract "0x366db0D543b709434Cb91113270521e50fC2fe49" --network Map
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

