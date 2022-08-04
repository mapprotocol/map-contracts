const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('../test/utils/borsh');
const hre = require("hardhat");
const { promisify } = require('util');
const { SourceLocation } = require('hardhat/internal/hardhat-network/stack-traces/model');
const sleep = promisify(setTimeout);
const { Web3 } = require('../test/utils/robust')



let initData = '0x8129fc1c';
let nearcms = '0x6d63732e70616e646172722e746573746e6574';
async function main() {

  let [wallet] = await hre.ethers.getSigners();

  const LightNode = await hre.ethers.getContractFactory("LightNode");
  const lightNode = await LightNode.connect(wallet).deploy();
  await lightNode.deployed();
  console.log("Implementation deployed to .....", lightNode.address);

  const LightNodeProxy = await hre.ethers.getContractFactory("LightNodeProxy");
  const lightNodeProxy = await LightNodeProxy.connect(wallet).deploy(lightNode.address, initData);
  await lightNodeProxy.deployed();
  console.log("lightNodeProxy deployed to .....", lightNodeProxy.address);

  const proxy = LightNode.attach(lightNodeProxy.address);

   let block = borshify(require('./data/block.json'));
   let validators = borshifyInitialValidators(require('./data/validators.json').next_bps);

   await (await proxy.connect(wallet).initWithValidators(validators, { gasLimit: 20000000 })).wait();

   await sleep(20000);
   
   await (await proxy.connect(wallet).initWithBlock(block, { gasLimit: 20000000 })).wait();

   await sleep(20000);

   await (await proxy.updateBlockHeader(borshify(require('./data/addBlock.json')), { gasLimit: 20000000 })).wait();

   await sleep(20000);

   console.log(await proxy.headerHeight());

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
