# MAP Cross-chain Service


## Setup Instructions
Edit the .env-example.txt file and save it as .env

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

## Instruction
FeeCenter contract is a contract used to manage cross-chain charges

MapCrossChainService contract is suitable for evm-compatible chains and implements cross-chain logic

MAPCrossChainServiceRelay contract implements cross-chain logic and basic cross-chain control based on MapChain

TokenRegister contract is used to control the mapping of cross-chain tokens

The MAPVaultToken contract is a treasury and fee growth for users to provide cross-chain pledges

StandardToken contract is a token contract that has roles to control minting and destruction

MapCrossChainServiceProxy is the contract for MapCrossChainService upgrade

MAPCrossChainServiceRelayProxy is the contract for MAPCrossChainServiceRelay upgrade

### Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/mcs/evm/
npm install
```

### Test it with the following command

```shell
npx hardhat test
```

### Follow the steps below to deploy

##### 1.We need some basic contract addresses, please fill in the correct contract address into the deployConfig.js file under the path of map-contracts/mcs/evm/deploy/config/

##### 2.If you do not have some Token contract addresses, you can also execute the following commands to redeploy these Tokens

````
npx hardhat deploy --tags WETH --network <network>
npx hardhat deploy --tags MapToken --network <network>
npx hardhat deploy --tags MakaluCrossChainToken --network <network>
````
Note that you need at least the token addresses of two chains. The network behind the command can replace the deployed network, and you can choose a different chain for deployment. For specific configuration, please refer to
map-contracts/mcs/evm/hardhat.config.js file

##### 3.Okay, the initial configuration is done, let's do the basic deployment of the cross-chain contract

Note you'll need some testnet funds in your wallet to deploy the contract.

The following four commands are generally applicable to Map mainnet and Makalu testnet
```
npx hardhat deploy --tags FeeCenter --network <network>
npx hardhat deploy --tags TokenRegister --network <network>
npx hardhat deploy --tags MAPCrossChainServiceRelay --network <network>
npx hardhat deploy --tags MAPVaultToken --network <network>
```

The following commands are for EVM compatible blockchains
```
npx hardhat deploy --tags MapCrossChainService --network <network>
```
##### 4.After we have completed the basic contract deployment, please correctly complete the initialConfig.js file configuration under the path of map-contracts/mcs/evm/deploy/config/

Please note that if you do not want to configure the Near chain information, do not configure the mcsNearChainId field information

##### 5.We execute the following command on the EVM compatible chain
```
npx hardhat deploy --tags MapCrossChainServiceProxySet --network <network>
```
The following command applies to the cross-chain contract configuration of Map mainnet and Makalu testnet
```
npx hardhat deploy --tags MAPCrossChainServiceRelayProxySet --network <network>
```
##### 6.If you want to configure the Near chain separately, use the following command, but make sure that the mcsNearChainId field and the nearExecuteId field in the initialConfig.js file under the path map-contracts/mcs/evm/deploy/config/ are configured correctly
```
npx hardhat deploy --tags MAPCrossChainServiceRelayProxyNearSet --network <network>
```
Note that sequence number 6 is not required
##### 7.Now let's complete the final configuration, please read the tokenCrossChainConfig.js file under the path map-contracts/mcs/evm/deploy/config/, fill in the configuration information, and execute the following command
Please execute the following command on the EVM compatible chain
```
npx hardhat deploy --tags MapCrossChainServiceSetToken --network <network>
```
##### 8.There is a final step, execute the following command to allow the mutual cross-chain completion of Token

The following command applies to the cross-chain contract configuration of Map mainnet and Makalu testnet
```
npx hardhat deploy --tags TokenCrossChainSet --network <network>
```


#### Above, we completed the deployment of a cross-chain contract and the cross-chain configuration of Token, but when we have a better cross-chain idea, we can upgrade the cross-chain contract through the following commands

Please execute the following command on the EVM compatible chain
```
npx hardhat deploy --tags MapCrossChainServiceProxyUp --network <network>
```
The following command applies to the cross-chain contract configuration of Map mainnet and Makalu testnet
```
npx hardhat deploy --tags MAPCrossChainServiceRelayProxyUp --network <network>
```


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
  function setBridgeAddress(uint256 _chainId, bytes memory _addr) external onlyManager {
      bridgeAddress[_addr] = _chainId;
  }
  ```