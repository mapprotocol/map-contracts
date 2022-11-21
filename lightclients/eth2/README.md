# Brief Description

The LightNode.sol contract is an implementation of eth2.0 light client on map chain. 
An off-chain program called maintainer retrieves LightClientUpdate from ethereum beacon chain and sends it to light client periodically.
The light client will verify the validity of the update. 
After then maintainer gets the ethereum execution layer block headers in the range of this update and the last update, and sends them to the light client.
The light client also verify these block headers' validity and store the hashes for further proof verification.

The LightNodeProxy.sol contract is an proxy contract of LightNode.sol.

The MPTVerify.sol contract is receipt MerklePatriciaProof verify util.

# Contract Deployment Workflow

## Pre-requirement

Since all the contracts are developed in Hardhat development environment, developers need to install Hardhat before working through our contracts. 
The hardhat installation tutorial can be found [here](https://hardhat.org/hardhat-runner/docs/getting-started#installation).

### install

```
npm install
```

Create an .env file and fill in the following contents.

```
# your private key on the blockchain the contracts will be deployed to
PRIVATE_KEY=
# beacon chain rpc url
URL=
# eth mainnet 1 Goerli 5
CHAIN_ID =
# initialize the light client with the information of the period
PERIOD=
# report gas or not
REPORT_GAS=
# default network to test or deploy, one of "hardhat", "local", "makalu", "dev" and "map"
DEFAULT_NETWORK=
```

## Compiling contracts

We can simply use hardhat built-in compile task to compile our contracts in the `contracts/` folder.

```
$ npx hardhat compile
Compiling...
Compiled 1 contract successfully
```

The compiled artifacts will be saved in the `artifacts/` directory by default

## Testing contracts

```
$ npx hardhat test
Compiled 19 Solidity files successfully


   LightNode
    Initialization
      ✔ initialization should be OK (3495ms)
      ✔ can not initialization sync committee after contract is initialized
      ✔ re-initialization should fail
      ✔ initialization with wrong sync committee keys should be fail (158ms)
    Upgrade
      ✔ Implementation upgrade must be admin (52ms)
      ✔ Implementation upgrade is OK (70ms)
    Permission check
      ✔ Change admin (41ms)
      ✔ togglePause  only admin  (80ms)
    Update light client
      ✔ updateLightClient ... paused  (2154ms)
      ✔ updateLightClient ... OK  (2625ms)
      ✔ updateLightClient should be failed when previous exe block headers are not updated  (3414ms)
    Update execution header
      ✔ updateExeBlockHeaders ... ok  (1294ms)
    Verify proof data
      ✔ verifyProofData ... ok  (1939ms)


  13 passing (15s)

```

## Deploy contracts

The deploy script is located in `deploy/` folder. We can run the following commands to deploy.

```
//deploy MPTVerify
npx hardhat deploy --tags MPTVerify

//deploy MPTVerify on makalu network
npx hardhat deploy --tags MPTVerify --network makalu

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

[more details about hardhat-depoly are available](https://github.com/wighawag/hardhat-deploy)

[makalu faucet ](https://faucet.maplabs.io/)
