const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('../test/utils/borsh');
const hre = require("hardhat");
const { promisify } = require('util');
const { SourceLocation } = require('hardhat/internal/hardhat-network/stack-traces/model');
const sleep = promisify(setTimeout);
const { Web3 } = require('../test/utils/robust')


//let nearProofProducerAccount_ = "0x6175726f7261"; //0x6d63732e70616e646172722e746573746e6574

let initData = '0x439fab91000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000066175726f72610000000000000000000000000000000000000000000000000000';
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

  await proxy.connect(wallet).setNearProofProducerAccount_(nearcms);

  let block = borshify(require('./data/block.json'));
  let validators = borshifyInitialValidators(require('./data/validators.json').next_bps);

  await proxy.connect(wallet).initWithValidators(validators, { gasLimit: 20000000 });

  await sleep(20000);
  await proxy.connect(wallet).initWithBlock(block, { gasLimit: 20000000 });

  await sleep(20000);

  await proxy.updateBlockHeader(borshify(require('./data/addBlock.json')), { gasLimit: 20000000 })

  await sleep(20000);

  console.log(await proxy.headerHeight());
  // await sleep(20000);
  //  const proxy = LightNode.attach("0xeA9066b735dA0ad462B269711be8e39fe7156d15");

  //  let block = "0x" + borshify(require('./data/addBlock.json')).toString('hex');

  //  let proof = "0x" + borshifyOutcomeProof(require('./data/proof.json')).toString('hex');

  // await proxy.updateBlockHeader(borshify(require('./data/addBlock.json')), { gasLimit: 20000000 })

  //  await sleep(20000);
  //  console.log(await proxy.curHeight());

  // let types = [
  //   'bytes',
  //   'bytes'
  // ]

  // let values = [
  //   block,
  //   proof
  // ]

  // await sleep(10000);

  // let result = await proxy.verifyProofData(ethers.utils.defaultAbiCoder.encode(types, values), { gasLimit: 20000000 });

  // console.log(result);

  //  console.log(Buffer.from('aurora', 'utf8').toString('hex'));
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
