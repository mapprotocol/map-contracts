# Brief Description

The LightNode.sol contract is an custom implementation of Near light client on map chain. It operate by periodically fetching instances of LightClientBlockView from Near and verify its validity.

The LightNodeProxy.sol contract is an proxy contract of LightNode.sol.

# Contract Deployment Workflow

## Pre-requirement

Since all of the contracts are developed in Hardhat development environment, developers need to install Hardhat before working through our contracts. The hardhat installation tutorial can be found here[hardhat](https://hardhat.org/hardhat-runner/docs/getting-started#installation)

### install

```
npm install
```

create an .env file and fill following in the contents

```
#your ethereum account private key
PRIVATE_KEY = 
// near rpc url
RPC_URL = 
//mainnet or testnet mainnet means syncing near mainnet light block,testnet means syncing testnet block,make sure rpc_url with the network is same
NETWORK = 
```

### Compiling contracts

We can simply use hardhat built-in compile task to compile our contract in the contracts folder.

```
$ npx hardhat compile
Compiling...
Compiled 1 contract successfully
```

The compiled artifacts will be saved in the `artifacts/` directory by default

## Testing contracts

Our test cases are separated into two files:

**LightNode.js** including test cases related to basic functions and some configuration functions

we can use hardhat basic test task to run through it:

(Make sure you have closed /test/oncjainTest.js before you begin)

```
$ npx hardhat test
Compiled 19 Solidity files successfully


  lightNode
    Deployment
Implementation deployed to ..... 0x5FbDB2315678afecb367f032d93F642f64180aa3
lightNodeProxy deployed to ..... 0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
      √ initWithValidators must owner (721ms)
      √ initWithBlock must owner (441ms)
      √ init should be ok (573ms)
      √ Implementation upgradle must admin (102ms)
      √ Implementation upgradle ok (128ms)
      √ change admin  (78ms)
      √ trigglePause  only admin  (111ms)
      √ updateBlockHeader ... paused  (504ms)


  8 passing (3s)

```

**onChainTest.js** including test cases related to proof validation. It also contains proof validation using map pre-compiled Ed25519 contracts.

This test requires a different setup since it is tested on makalu([faucet](https://faucet.maplabs.io/)).

The result will be printed out with a true value in success field.

All the data used in test cases are stored in data folder and they are extracted from Near blockchain history transactions.

(Make sure you have opened /test/oncjainTest.js before you begin)

```
npx hardhat run ./test/onChainTest.js --network makalu
```

## Deploy contracts

before deploy you can refresh init data by run the following command

```
npx hardhat run ./scripts/refresh.js
```

It may fail, but it doesn't matter. Try it again.It also doesn't matter if it never works out,you can still

deploy.

The deploy script is located in deploy folder . We can run the following command to deploy.

```
//deploy lightNode implementation
npx hardhat deploy --tags LightNode

//deploy lightNode implementation on makalu network
npx hardhat deploy --tags LightNode --network makalu

//deploy lightNode proxy 
npx hardhat deploy --tags Proxy

//deploy lightNode proxy on makalu network
npx hardhat deploy --tags Proxy --network makalu

// upgrade 
npx hardhat deploy --tags Upgrade --network makalu
```

[more details about hardhat-deploy are available](https://github.com/wighawag/hardhat-deploy)

[makalu faucet ](https://faucet.maplabs.io/)
