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

##### 1.We need some basic contract addresses, if not, please use the command in the second step to deploy

##### 2.The following commands can deploy some cross-chain tokens, please use the correct parameters and network to deploy

````
npx hardhat deploy --tags WETH --network <network>
npx hardhat deployCrossToken --name <token name > --symbol <token symbol> --network <network>
````
Note that if you need multiple tokens, you can execute the second command multiple times by changing the parameters

##### 3.We have deployed the basic token contract, now we will deploy the cross-chain contract
Note you'll need some testnet funds in your wallet to deploy the contract.

The following four commands are generally applicable to Map mainnet and Makalu testnet
```
npx hardhat deploy --tags FeeCenter --network <network>
npx hardhat deploy --tags TokenRegister --network <network>
npx hardhat deploy --tags MAPVaultToken --network <network>
```
````
npx hardhat deployMapCrossChainServiceRelayProxy --weth <weth address> --maptoken <maptoken address> --lightnode <lightNodeManager address> --network <network>
````
The following commands are for EVM compatible blockchains
```
npx hardhat deployMapCrossChainServiceProxy --weth <weth address> --maptoken <maptoken address> --lightnode <lightnode address> --network <network>
```
##### 4.Ok, now our cross-chain contract is basically deployed, let me do some basic settings of the contract

The following command on the EVM compatible chain
```
npx hardhat mapCrossChainServiceSet --relayaddress <mapCrossChainServiceRelay address> --chainid <map chainId> --network <network>
```
The following command applies to the cross-chain contract configuration of Map mainnet and Makalu testnet
```
npx hardhat mapCrossChainServiceRelaySet --feecenter <feeCenter address> --registertoken <registertoken address> --network <network>
npx hardhat mapCrossChainServiceRelaySetBridgeAddress --mcsaddr <mapCrossChainService address> --mcsid <mapCrossChainService chainId> --network <network>
```
##### 5.If you want to use the near chain, use the following command to configure the near chain
The following commands are for EVM compatible blockchains
```
npx hardhat mapCrossChainServiceInitNear --nearid <near chainId> --network <network>
```
The following four commands are generally applicable to Map mainnet and Makalu testnet
```
npx hardhat mapCrossChainServiceRelayInitNear --nearid <near chainId> --network <network>
```
Note that sequence number 5 is not required

##### 6.Earlier we made a basic configuration of a cross-chain contract. Next, we will do a cross-chain setup of a token.
Note the following four commands are generally applicable to Map mainnet and Makalu testnet
1. Bind the token address on the map chain to the vault and initialize it, and execute the following command
````
npx hardhat vaultTokenInit --correspond <mapchain mapping token address> --vaultname <vault token name> --vaultsymbol <vault token symbol> --network <network>
````
2. FeeCenter sets up the treasury and token binding
````
npx hardhat feeCenterSetTokenVault --vaulttoken <vault address> --crosstoken <mapchain mapping token address> --network <network>
````
3. FeeCenter sets fee distribution
````
npx hardhat feeCenterSetDistributeRate --vaulttoken <vault address> --ratenumber <rate 0-10000> --network <network>
````
4. FeeCenter sets the token cross-chain fee ratio
````
npx hardhat feeCenterSetChainTokenGasFee --mcschainid <MapCrossChainService chainId> --crosstoken <mapchain mapping token address> --minfee <minimum value> --maxfee <maximum value> --ratefee <rate 0-10000> --network <network>
````
5. Bind the token mapping relationship between the two chains that requires cross-chain
````
npx hardhat tokenRegister --crossid <cross-chain id> --crosstoken <cross-chain token> --maptoken <mapchain mapping token address> --network <network>
````
6. MapCrossChainServiceRelay sets the decimal for cross-chain tokens
Note the mcsids and tokendecimals parameters can be filled with one or more words separated by ',' (eg 1,2,96 18,18,24)
````
npx hardhat mapCrossChainServiceRelaySetTokenDecimals --tokenaddress <token address> --mcsids <Multiple chainIds (1,2,96)> --tokendecimals <token decimals (18,18,24)> --network <network>
````
7. MapCrossChainServiceRelay sets the quota for cross-chain tokens to other chains
````
npx hardhat mapCrossChainServiceRelaySetVaultBalance --mcsid <MapCrossChainService chainId> --tokenaddress <token address> --tokennumber <Cross-chain quota> --network <network>
````

##### 7.There is a final step, execute the following command to allow the token of MapCrossChainService to cross-chain to other chains
Note the ids field allows multiple chainIds to be separated by ',' (eg 1, 2, 96, 58)
```
npx hardhat mapCrossChainServiceSetCanBridgeToken --tokenaddress <token address> --ids <cross-chain id> --network <network>
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