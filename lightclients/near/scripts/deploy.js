const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('../test/utils/borsh');
const hre = require("hardhat");
const { promisify } = require('util');
const sleep = promisify(setTimeout);


//let nearProofProducerAccount_ = "0x6175726f7261";

let initData = '0x439fab91000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000066175726f72610000000000000000000000000000000000000000000000000000';

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

  await proxy.connect(wallet).initWithValidators(validators, { gasLimit: 20000000 });

  await sleep(20000);
  await proxy.connect(wallet).initWithBlock(block, { gasLimit: 20000000 });

  await sleep(20000);
  //const proxy = LightNode.attach("0xC486E32DacC89F8c56abFE4a265E5193ABCe1ed6");
  await proxy.updateBlockHeader(borshify(require('./data/addBlock.json')), { gasLimit: 20000000 })

  // console.log(borshify(require('./data/addBlock.json')).toString('hex'));

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
