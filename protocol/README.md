# Map chain protocol

## Setup Instructions
Edit the .env-example.txt file and save it as .env

Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd protocol
npm install
```

From there we can test and deploy

```shell
npx hardhat test
```

Note you'll need some testnet funds in your wallet to deploy the contract.

```shell
npx hardhat deploy --tags MaintainerManager --network Map
```


The contracts main

```shell
MaintainerManager 0x366db0D543b709434Cb91113270521e50fC2fe49
MaintainerManagerProxy 0x29B1bC29634e630967cf1A74973Bd0047880bE89
```