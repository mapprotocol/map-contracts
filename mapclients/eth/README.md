
# Map chain light client, deployed on EVM chains.

## Setup Instructions
Edit the .env-example.txt file and save it as .env

Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/mapclients/eth/
npm install
```

From there we can test and deploy

```shell
npx hardhat test
```

Note you'll need some testnet funds in your wallet to deploy the contract.

```shell
npx hardhat deploy --tags LightNode --network Map
```
