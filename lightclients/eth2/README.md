# Eth2.0 PoS light client

**Eth2.0 PoS light client which will be deployed on MAP blockchain.**

## Table of Contents

- [Pre-requisites](#pre-requisites)
- [Compile the contracts](#compile-the-contracts)
- [Test the contracts](#Test the contracts)
- [Deploy the contracts](#deploy-the-contracts)
- [Main interfaces explanation](#main-interfaces-explanation)
- [More reference](#more-reference)

## Pre-requirement

1. First you need to install **npm** and **node**,
   click [here](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) to see how to install them.
2. Since all the contracts are developed in **Hardhat** development environment, developers need to install Hardhat
   before working through our contracts.
   The hardhat installation tutorial can be
   found [here](https://hardhat.org/hardhat-runner/docs/getting-started#installation).

After that, run below command to install dependencies.

```
npm install
```

## Compile the contracts

We can simply use hardhat built-in compile task to compile our contracts in the `contracts/` folder.

```
$ npx hardhat compile
Compiled 19 Solidity files successfully
```

The compiled artifacts will be saved in the `artifacts/` directory by default

## Test the contracts

Run below command to run the test cases.

```
$ npx hardhat test
Compiled 19 Solidity files successfully


   LightNode
    Initialization
      ✔ initialization should be OK (3495ms)
      ✔ can not initialization sync committee after contract is initialized
      ✔ re-initialization should fail
      ✔ initialization with wrong sync committee keys should be fail (158ms)
    Upgrade
      ✔ Implementation upgrade must be admin (52ms)
      ✔ Implementation upgrade is OK (70ms)
    Permission check
      ✔ Change admin (41ms)
      ✔ togglePause  only admin  (80ms)
    Update light client
      ✔ updateLightClient ... paused  (2154ms)
      ✔ updateLightClient ... OK  (2625ms)
      ✔ updateLightClient should be failed when previous exe block headers are not updated  (3414ms)
    Update execution header
      ✔ updateExeBlockHeaders ... ok  (1294ms)
    Verify proof data
      ✔ verifyProofData ... ok  (1939ms)


  13 passing (15s)

```

## Deploy contracts

Create an .env file and fill in the following contents.

```
# your private key on the MAP blockchain the contracts will be deployed to
PRIVATE_KEY=
# beacon chain rpc url
URL=
# eth mainnet 1 Goerli 5
CHAIN_ID =
# initialize the light client base on the trusted root
TRUSTED_BLOCK_ROOT=
```

The deployment script is located in `deploy/` folder. We can run the following commands to deploy.

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

## Main interfaces explanation

The file LightNode.sol implements the main logic of eth2.0 PoS light client. Here are some important public interfaces.

* Below 2 functions are used for contract initialization. The scripts in `/deploy` get the light client bootstrap
  information based on the trusted block root through beacon chain RPC, also the scripts get some other necessary
  information, and then call these 2 functions to initialize the light client.

```solidity
    function initialize(
    uint64 _chainId,
    address _controller,
    address _mptVerify,
    BeaconBlockHeader memory _finalizedBeaconHeader,
    uint256 _finalizedExeHeaderNumber,
    bytes32 _finalizedExeHeaderHash,
    bytes memory curSyncCommitteeAggPubKey,
    bytes memory nextSyncCommitteeAggPubKey,
    bytes32[] memory syncCommitteePubkeyHashes,
    bool _verifyUpdate
) public initializer;

    function initSyncCommitteePubkey(bytes memory syncCommitteePubkeyPart) public;

```

* An off-chain program called maintainer will get the update information through beacon chain RPC and execution layer
  blockchain RPC periodically, and call the below interface to update the light client. The light client will verify the
  update information and save them if verification is passed.

```solidity
    function updateLightClient(LightClientUpdate memory update) external override whenNotPaused;

```

* Everytime the maintainer calls the above interface to update the light client successfully, the finalized beacon
  header number and finalized execution layer header number are updated. There is a gap between the latest finalized
  execution layer header and the previous finalized header. The maintainer should call below method to update the block
  headers one or more times, and the light client will verify the validity of the block headers and store the hashes
  for further proof verification. The maintainer could not do the next light client update util the gap is filled.

```solidity
    function updateExeBlockHeaders(BlockHeader[] memory headers);
```

* Below interface `verifyProofData` can verify the validity of the receipt and logs of a transaction in ethereum
  execution layer
  block if it's hash has already been stored in the light client through the above interface `updateExeBlockHeaders`.

```solidity
    function verifyProofData(bytes memory receiptProof)
    external
    view
    override
    returns (
        bool success,
        string memory message,
        bytes memory logs
    );
```

* Below interface `verifiableHeaderRange` returns the range of the execution layer block header number between which
  you can verify the proof through the above interface `verifyProofData`.

```solidity
    function verifiableHeaderRange() external view returns (uint256, uint256);
```
* When the maintainer starts, it should call first get the public state `exeHeaderUpdateInfo` which tells if the latest
 execution layer headers' gap is filled. If not, the maintainer should first call `updateExeBlockHeaders` to fill the gap.
 Then call below interface `finalizedSlot` to get the latest finalized slot of the light client. So that it can decide
 whether to retrieve the period update information or the latest finality update information, and then call 
 `updateLightClient` to update the light client.
```solidity
    ExeHeaderUpdateInfo public exeHeaderUpdateInfo;

    struct ExeHeaderUpdateInfo {
        uint256 startNumber;
        uint256 endNumber;
        bytes32 endHash;
    }

    function finalizedSlot() external view override returns (uint256);
```

## More reference

[more details about hardhat-depoly are available](https://github.com/wighawag/hardhat-deploy)

[makalu faucet ](https://faucet.maplabs.io/)
