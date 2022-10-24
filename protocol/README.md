# Map chain protocol

## Introduction
`LightClientManager` is a contract on Map chain used to manage contract addresses and calls on other chains.

`MaintainerManager` is a contract on Mapchain for managing all Maintainer pledges, reward distribution, and binding work accounts

`MaintainerManagerUp` is the contract for MaintainerManager upgrade

## Configuration file description

`PRIVATE_KEY` User-deployed private key

`INFURA_KEY` User-deployed infura key

## Compile

Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd protocol
npm install
```

Edit the .env-example.txt file and save it as .env

## Test

```shell
npx hardhat test
```

## Deploy

```shell
npx hardhat deploy --tags MaintainerManager --network Map

npx hardhat deploy --tags LightClientManager --network Map
```

## Upgrade

```shell
npx hardhat deploy --tags MaintainerManagerUp --network Map
```


## Parameter setting

### LightClientManager contract

#### Add a chain of LightNode contracts

cmd
```shell
npx hardhat LightClientRegister --chain <chain id for light client> --contract <contract for light client>  --network <network>
```
example
```shell
  npx hardhat LightClientRegister --chain 1 --contract "0x366db0D543b709434Cb91113270521e50fC2fe49" --network Map
```

### MaintainerManager contract

#### Add or remove a Maintainer

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