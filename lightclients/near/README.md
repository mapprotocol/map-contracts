# Brief Description

The LightNode.sol contract is an custom implementation of Near light client on map chain. It operate by periodically fetching instances of LightClientBlockView from Near and verify its validity.

The LightNodeProxy.sol contract is an proxy contract of LightNode.sol.



# Contract Deployment Workflow

## Pre-requirement

Since all of the contracts are developed in Hardhat development environment, developers need to install Hardhat before working through our contracts. The hardhat installation tutorial can be found here 

[Hardhat]: https://hardhat.org/hardhat-runner/docs/getting-started#installation	"Hardhat installation"



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

**LightNode.sol** including test cases related to basic functions and some configuration functions

we can use hardhat basic test task to run through it:

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

**onChainTest.sol** including test cases related to proof validation. It also contains proof validation using map pre-compiled Ed25519 contracts.

This test requires a different setup since it is tested on map testnet.  

We need to first enter a funded account mnemonic in .env file.

```
MNEMONIC = test test orphan test illegal father test pupil test forward mammal cinnamon
```

Then we need to add custom map_test network settings in hardhat config file.

```json
 networks: {
    ropsten: {
      url: process.env.ROPSTEN_URL || "",
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    map_test: {
      chainId: 212,
      url: process.env.ROPSTEN_URL || "http://18.142.54.137:7445",
      accounts: { mnemonic: process.env.MNEMONIC},
    },
    
  },
```

The result will be printed out with a true value in success field.



All the data used in test cases are stored in data folder and they are extracted from Near blockchain history transactions.



## Deploy contracts

The deploy script is located in script folder and is named deploy.js. We can run the following command to target deployment on map testnet which is added in Hardhat config.

```
npx hardhat run --network map_test scripts/deploy.js
```



​	