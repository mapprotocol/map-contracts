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



MAP Cross Chain Service parameter setting

- Setting a Token can be cross-chain or canceled

```solidity
function setCanBridgeToken(address token, uint chainId, bool canBridge) public 	onlyManager {
        canBridgeToken[token][chainId] = canBridge;
    }
```



MAP Cross Chain Service Relay parameter setting

- Set cross-chain fees for FeeCenter contracts

  ```solidity
  function setChainTokenGasFee(uint to, address token, uint lowest, uint highest,uint proportion) external onlyManager {
      chainTokenGasFee[to][token] = gasFee(lowest,highest,proportion);
  }
  ```

- Set the decimals of the cross-chain token

  ```solidity
  function setTokenOtherChainDecimals(bytes memory selfToken, uint256 chainId, uint256 decimals) external onlyManager {
      tokenOtherChainDecimals[selfToken][chainId] = decimals;
  }
  ```

- Set the number of cross-chain vault tokens (if needed)

  ```solidity
  function setVaultBalance(uint256 tochain, address token, uint256 amount) external onlyManager {
      vaultBalance[tochain][token] = amount;
  }
  ```

- Set other chain msc contract address for verification

  ```solidity
  function setBridageAddress(uint256 _chainId, bytes memory _addr) external onlyManager {
      bridgeAddress[_addr] = _chainId;
  }
  ```