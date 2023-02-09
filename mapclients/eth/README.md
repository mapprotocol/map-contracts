
# Mapo relay chain light client, deployed on EVM chains.



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
LightNode contracts for EVM compatible chain sync block and transaction verification

The VerifyTool contract is suitable for parsing the data of the LightNode contract and provides some reliable methods

LightNodeProxy is the contract for LightNode upgrade


### Build using the following commands:

```shell
git clone https://github.com/mapprotocol/map-contracts.git
cd map-contracts/mapclients/eth/
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
LightNode start test
    √ deploy LightNode
    √ initialize 
    √ deploy LightNodeProxy
    √ get test
    √ updateBlockHeader and verifyProofData
    √ add validator
    √ authorizeUpgrade test 
    √ delete deploy
    √ verifyProofData error test
    
     9 passing (15s)
```
### Deploy it with the following command
When you want to deploy LightNode, first run the following command
````
npx hardhat initializeData --epoch <epoch>  --network <network>
````
After the execution is successful, check whether the content of the map-contracts/mapclients/eth/deploy/config.js file is similar to the following
````
map-contracts/mapclients/eth/deploy/config.js

let initData ={"epoch":12,"epoch_size":1000,"threshold":3,"validators":[{"weight":1,"address":"0x053af2b1ccbacba47c659b977e93571c89c49654","g1_pub_key":{"x":"0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c1","y":"0x2b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287"}},{"weight":1,"address":"0xb47adf1e504601ff7682b68ba7990410b92cd958","g1_pub_key":{"x":"0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a69","y":"0x2685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7"}},{"weight":1,"address":"0xf655fc7c95c70a118f98b46ca5028746284349a5","g1_pub_key":{"x":"0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835","y":"0x156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042"}},{"weight":1,"address":"0xb243f68e8e3245464d21b79c7ceae347ecc08ea6","g1_pub_key":{"x":"0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b","y":"0x131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37"}}]}
module.exports = initData
````
When using the following command network refers to the name of the network you need to deploy. See networks configuration in **hardhat.config.js** file for details
Note you'll need some testnet funds in your wallet to deploy the contract.

```shell
npx hardhat deploy --tags LightNode --network <network> 
```

If you want to use the upgrade contract, please execute the following command, pay attention to use the correct network

```shell
npx hardhat deploy --tags LightNodeUp --network <network> 
```
