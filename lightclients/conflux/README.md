
# Conflux chain light client.



The following node and npm versions are required
````
$ node -v
v14.17.1
$ npm -v
6.14.13
````
Configuration file description

PRIVATE_KEY User-deployed private key

LIGHTNODE_SALT User-deployed lightnode salt

DEPLOY_FACTORY Factory-contract address


## Instruction
LightNode : A contract used to verify conflux transactions in the map relay chain

LightNodeProxy is the contract for LightNode upgrade


### Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/lightlients/conflux/
npm install
```

Edit the .env-example.txt file and save it as .env

### Compile with the following command

```shell
npx hardhat compile
```

### Test it with the following command

```shell
npx hardhat test
```

See results similar to the following, proving that the execution was successful

```shell
  LightNode start test
Init epoch:  15476
    √ check admin test (1964ms)
    √ updateLightClient test (13027ms)
    √ updateBlockHeader test (3289ms)
Update epoch:  15477
CurrentFinalizedBlockNumber: 137851620
    √ ligntnode upgradle updateLightClient and updateBlockHeader (39863ms)


  4 passing (58s)


```
### Deploy it with the following command
When you want to deploy LightNode, first run the following command
1. Deploy LedgerInfo
```
npx hardhat deploy --tags LedgerInfo --network <network>
```

2. Deploy Provable
```
npx hardhat deploy --tags Provable --network <network>
```

3. Deploy LightNode
````
npx hardhat deploy --tags LightNode --network <network>
````

Later we deploy the upgrade contract and initialize the contract，run the following command

````
npx hardhat lightClientDeploy --mpt <Provable contract address> --ledger <LedgerInfo contract address> --network  <network>
````

If you want to use the upgrade contract, please execute the following command, pay attention to use the correct network

```shell
npx hardhat deploy --tags LightNodeUp --network <network> 
```
