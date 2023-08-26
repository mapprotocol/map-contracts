# MAP Protocol management contracts

## Introduction


### Deploy Factory

The factory contract helps to deploy to deterministic addresses without an init code factor.
Every developer can use contract `0x6258e4d2950757A749a4d4683A7342261ce12471` to deploy deterministic addresses contract.

Read [this](create3/README.md) to get more information.

### MPT verifier
The contract deployed on the MAP Relay Chain is responsible for MPT proof verify.

MAPO mainnet address: `0x4b1EE84A72b44B78346e069D1c66509940827E22`
Makalu testnet address: `0x4b1EE84A72b44B78346e069D1c66509940827E22`

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