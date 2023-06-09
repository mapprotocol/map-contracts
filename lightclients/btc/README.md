
# Btc chain light client.

The following node and npm versions are required
````
$ node -v
v14.17.1
$ npm -v
6.14.13
````
Configuration file description

PRIVATE_KEY User-deployed private key

INFURA_KEY User-deployed infura key

LIGHTNODE_BTC_SALT User-deployed btc lightnode salt

DEPLOY_FACTORY Factory-contract address


## Instruction
LightNode : A contract used to verify btc transactions in the mapo chain

LightNodeProxy is the contract for LightNode upgrade


### Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/lightlients/btc/
npm install
```

Note if you use the eth main network, you can configure ETH_INFURA_KEY, if you don't use it, you can not configure it
Edit the .env-example.txt file and save it as .env


### Test it with the following command

```shell
npx hardhat test
```

See results similar to the following, proving that the execution was successful

```shell
  LightNode contract start test
    LightNode updateBlockHeader test start
      √ updateBlockHeader test (1248ms)
      √  updateBlockHeader error test  (63ms)
    LightNode verifyProofData test start
      √ verifyProofData test (126ms)


  3 passing (1s)
```
### Deploy it with the following command
When you want to deploy LightNode, first run the following command
````
npx hardhat deploy --tags LightNode --network <network>
````
Later we deploy the upgrade contract and initialize the contract，run the following command

````
npx hardhat lightFactoryDeploy --height <init height> --header <btc block chain block header> --network  <network>

````

If you want to use the upgrade contract, please execute the following command, pay attention to use the correct network

```shell
npx hardhat deploy --tags LightNodeUp --network <network> 
```
