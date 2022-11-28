# MAP Omnichain Service


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
MAPOmnichainServiceV2 contract is suitable for evm-compatible chains and implements cross-chain logic

MAPOmnichainServiceRelayV2 contract implements cross-chain logic and basic cross-chain control based on MAP Relay Chain

TokenRegisterV2 contract is used to control the mapping of cross-chain tokens

## Build

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/mcs/evmv2/
npm install
```

## Test

```shell
npx hardhat test
```



## Deploy

### MOS Relay
The following steps help to deploy MOS relay contracts on Map mainnet or Makalu testnet

1. Deploy Fee Center and Token Register
```
npx hardhat deploy --tags TokenRegister --network <network>
````
2. Deploy MOS Relay

```
npx hardhat relayDeploy --wrapped <wrapped token> --lightnode <lightNodeManager address> --network <network>
````

* `wrapped token` is wrapped MAP token address on MAP mainnet or MAP Makalu.
* `lightNodeManager address` is the light client mananger address deployed on MAP mainnet or MAP Makalu. See [here](../protocol/README.md) for more information.

3. Init MOS Relay
```
npx hardhat relayInit  --tokenmanager <token register address> --network <network>
````


4.  sets fee distribution
````
npx hardhat managerSetDistributeRate --type <0 to the token vault, 1 to specified receiver> --address <fee receiver address> --rate <rate 0-1000000, uni 0.000001> --network <network>
````

### MOS on EVM Chains

1. Deploy
```
npx hardhat mosDeploy --wrapped <native wrapped address> --lightnode <lightnode address> --network <network>
```

2. Set MOS Relay Address
   The following command on the EVM compatible chain
```
npx hardhat mosSetRelay --relay <Relay address> --chain <map chainId> --network <network>
```

3. Register
   The following command applies to the cross-chain contract configuration of Map mainnet and Makalu testnet
```
npx hardhat relayRegisterChain --address <MAPOmnichainService address> --chain <chain id> --network <network>
```

### MOS on other chain

The following four commands are generally applicable to Map mainnet and Makalu testnet
```
npx hardhat relayRegisterChain --address <MAPOmnichainService address> --chain <near chain id> --type 2 --network <network>
```
**NOTE**: Near Protocol testnet chain id 5566818579631833089, mainnet chain id 5566818579631833088

## Configure

### Deploy Token

1. Deploy a mintable Token
   If want to transfer token through MOS, the token must exist on target chain. Please depoly the mapped mintable token on target chain if it does NOT exist.
````
npx hardhat tokenDeploy --name <token name > --symbol <token symbol> --network <network>
````

2. Grant Mint Role to relay or mos contract
````
npx hardhat tokenGrant --token <token address > --minter <adress/mos> --network <network>
````

### Register Token


1. Relay Chain deploy vault token
Every token has a vault token. The vault token will distribute to the users that provide cross-chain liquidity.
The mos relay contract is manager of all vault tokens.

````
npx hardhat vaultDeploy --token <relaychain token address> --name <vault token name> --symbol <vault token symbol> --network <network>

npx hardhat vaultAddManager --vault <vault token address> --manager <manager address> --network <network>
````

2. Register token
````
npx hardhat relayRegisterToken --token <relaychain mapping token address> --vault <vault token address> --mintable <true/false> --network <network>
````

3. Set fee ratio to relay chain
```
npx hardhat relaySetTokenFee --token <token address> --chain <relay chain id>  --min <minimum fee value> --max <maximum fee value> --rate <fee rate 0-1000000> --network <network>
```

### Add Cross-chain Token

1. Relay Chain Bind the token mapping relationship between the two chains that requires cross-chain
````
npx hardhat relayMapToken --token <relay chain token address> --chain <cross-chain id> --chaintoken <cross-chain token> --decimals <cross-chain token decimals> --network <network>
````

2. Relay Chain sets the token cross-chain fee ratio
````
npx hardhat relaySetTokenFee --token <token address> --chain <chain id>  --min <minimum fee value> --max <maximum fee value> --rate <fee rate 0-1000000> --network <network>
````

3. Altchain sets token mintable
   
````
npx hardhat mosSetMintableToken --token <token address> --mintable <true/false> --network <network>
````

**NOTE:** If set the token mintable, the token must grant the minter role to mos contract.

4. Altchain sets bridge token

````
npx hardhat mosRegisterToken --token <token address> --chains < chain ids,separated by ',' > --network <network>
````



## Upgrade

When upgrade the mos contract through the following commands.

Please execute the following command on the EVM compatible chain

```
npx hardhat deploy --tags MAPOmnichainServiceV2Up --network <network>
```

Please execute the following command on relay chain mainnet or Makalu testnet
```
npx hardhat deploy --tags MAPOmnichainServiceRelayV2Up --network <network>
```

## Token cross-chain transfer deposit

1. token transfer
```
npx hardhat transferOutToken --mos <mos or relay address> --token <token address> --address <receiver address> --value <transfer value> --chain <chain id> --network <network>
```

2. token depsit
```
npx hardhat depositOutToken --mos <mos address> --token <token address> --address <receiver address> --value <transfer value> --network <network>
```

Note that the --token parameter is optional, if not set, it means to transfer out Native Token.
Similarly --address is also an optional parameter. If it is not filled in, it will be the default caller's address.

transfer native token to other chain:
```
npx hardhat depositOutToken --mos <mos or relay address>  --address <receiver address> --value <transfer value> --network <network>
```

transfer native token to sender's address:
```
npx hardhat transferOutToken --mos <mos or relay address> --value <transfer value> --chain <chain id> --network <network>
```


## List token mapped chain

1. relay chain
```
npx hardhat relayList --relay <relay address> --token <token address> --network <network>
```

2. altchains
```
npx hardhat mosList --mos <relay address> --token <token address> --network <network>
```