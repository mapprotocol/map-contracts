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
open `LightClientManagerSet.js`

change parameters
```js
 let chainId = 0; //chain id to add
 let contract =""; //lightNode contract to add
```
run

```shell
 npx hardhat deploy --tags LightClientManagerSet --network Map
```

### MaintainerManager contract

#### Add or remove a Maintainer

open `MaintainerManagerSet.js`

change parameters
```js
 let maintainer = ""; //maintainer address to add or remove
 let add = true;      //add as true remove as false
```
run

```shell
 npx hardhat deploy --tags MaintainerManagerSet --network Map
```

## The contracts main

```shell
MaintainerManager 0x366db0D543b709434Cb91113270521e50fC2fe49
MaintainerManagerProxy 0x29B1bC29634e630967cf1A74973Bd0047880bE89
```