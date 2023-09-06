## Brief Description

The LightNode.sol contract is an custom implementation of BSC light client on map chain. [basis for implementation](https://docs.bnbchain.org/docs/learn/consensus)

The LightNodeProxy.sol contract is an proxy contract of LightNode.sol.

The MPTVerify.sol contract is receipt MerklePatriciaProof verify util.

## Main interfaces explanation

the light node implementation principle is to verify the legitimacy of the block header by tracking validatorSet changes.

If we want to validate a transaction, we need to validate the block header that the transaction is in,to validate a block header and we need to validate the signature of the block header.

by tracking validatorSet changes light node can verify all bsc transations.

Here are some important public interfaces.

* Initialize the light client

  ```solidity
   function initialize(
          uint256 _chainId,
          uint256 _minEpochBlockExtraDataLen,
          address _controller,
          address _mptVerify,
          Verify.BlockHeader[2] memory _headers
      ) public initializer 
  ```

  to  set _chainId for bsc mainnet or testnet ,  and  pre set two consecutively epoch validatorSet. this initialization  data can verify everyone.
* syncing block header

  ```solidity

   function updateBlockHeader(bytes memory _blockHeadersBytes)
          external
          override
          whenNotPaused
  // _blockHeadersBytes: abi.encode(_blockHeaders)  BlockHeader[] memory _blockHeaders

  ```

  submit epoch block header to keep track of validatorSet changes for each epoch.To prove epoch block header legitimacy, (validatorSet length / 2)  block headers need to be submitted consecutively.  If consecutive blockheaders are signed by different signers in validatorSet we don't believe it was forged
* verify transation receipt

  ```solidity
  struct ProofData {
          Verify.BlockHeader[] headers;
          Verify.ReceiptProof receiptProof;
   }
  function verifyProofData(bytes memory _receiptProof)
          external
          view
          override
          returns (
              bool success,
              string memory message,
              bytes memory logs
          )
  // _receiptProof: abi.encode(_receiptProof)  ProofData memory _proof
  ```

  verify transation receipt and return receipt logs if succeed.
* get verifiable range

  ```solidity
  function verifiableHeaderRange() external view returns (uint256, uint256);

  ```

  returns the range of the execution layer block header number between which
  you can verify the proof through the above interface `verifyProofData`.

# Contract Deployment Workflow

## Pre-requirement

Since all of the contracts are developed in Hardhat development environment, developers need to install Hardhat before working through our contracts. The hardhat installation tutorial can be found here[hardhat](https://hardhat.org/hardhat-runner/docs/getting-started#installation)

### install

```
npm install
```

create an .env file and fill following in the contents

```
PRIVATE_KEY = 
# bsc rpc url
RPCURI = 
# mainnet 199 testnet 1028
CHAINID = 
// 0 - for latest
START_SYNCY_BLOCK = 0
# bttc mainnet 417 
MinEpochBlockExtraDataLen = 417
```

### Compiling contracts

We can simply use hardhat built-in compile task to compile our contract in the contracts folder.

```
$ npx hardhat compile
Compiling...
Compiled 1 contract successfully
```

The compiled artifacts will be saved in the `artifacts/` directory by default

## Testing contracts

```
$ npx hardhat test
 LightNode
    Deployment
      ✔ initBlock() -> correct (1346ms)
      ✔ upgradeTo() -> reverts only Admin (67ms)
      ✔ upgradeTo() -> correct (117ms)
      ✔ changeAdmin() -> reverts only Admin (40ms)
      ✔ changeAdmin() -> reverts for zero address
      ✔ changeAdmin() -> correct  (90ms)
      ✔ togglePause() -> reverts  only admin 
      ✔ togglePause() -> correct  (57ms)
      ✔ updateBlockHeader() -> reverts paused  (71ms)
      ✔ updateBlockHeader() -> correct  (631ms)
      ✔ verifyProofData() -> correct  (643ms)


  11 passing (3s)

```

## Deploy contracts

The deploy script is located in deploy folder . We can run the following command to deploy.

```
//deploy MPTVerify
npx hardhat deploy --tags MPTVerify

//deploy MPTVerify on makalu network
npx hardhat deploy --tags MPTVerify --network makalu

//deploy lightNode implementation
npx hardhat deploy --tags LightNode

//deploy lightNode implementation on makalu network
npx hardhat deploy --tags LightNode --network makalu

//deploy lightNode proxy 
npx hardhat deploy --tags Proxy

//deploy lightNode proxy on makalu network
npx hardhat deploy --tags Proxy --network makalu

// upgrade 
npx hardhat deploy --tags Upgrade --network makalu
```

[more details about hardhat-depoly are available](https://github.com/wighawag/hardhat-deploy)

[makalu faucet ](https://faucet.maplabs.io/)
