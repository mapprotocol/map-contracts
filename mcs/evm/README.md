# MAP Cross-chain Service


## Setup Instructions
Edit the .env-example.txt file and save it as .env

Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/mcs/evm/
npm install
```

From there we can test and deploy

```shell
npx hardhat test
```

Note you'll need some testnet funds in your wallet to deploy the contract.

```shell
npx hardhat deploy --tags MapCrossChainService --network ETH
npx hardhat deploy --tags MAPCrossChainServiceRelay --network MAP
```

Note when you deploy the MapCrossChainService contract,need a LightNode contract on the chain you are testing,will deploy/MapCrossChainService lightNodeAddress field replacement of files