const hre = require("hardhat");
const { ethers } = require("hardhat");
const { promisify } = require('util');
const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('./utils/borsh');
const sleep = promisify(setTimeout);

let nearcms = '0x6d63732e70616e646172722e746573746e6574';
let initData = '0x439fab91000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000066175726f72610000000000000000000000000000000000000000000000000000';
async function main() {

  await verifyProofData();

}



async function verifyProofData() {
  let [wallet] = await ethers.getSigners();


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

  let head = "0x" + borshify(require('./data/proofHead.json')).toString('hex');
  let proof = "0x" + borshifyOutcomeProof(require('./data/proof.json')).toString('hex');

  let types = [
    'bytes',
    'bytes'
  ]

  let values = [
    head,
    proof
  ]

  await sleep(30000);

  let result = await proxy.verifyProofData(ethers.utils.defaultAbiCoder.encode(types, values), { gasLimit: 20000000 });

  console.log(result);

  let proof2 = "0x" + borshifyOutcomeProof(require('./data/proof2.json')).toString('hex');

  values = [
    head,
    proof2
  ]

  await sleep(30000);
  result = await proxy.verifyProofData(ethers.utils.defaultAbiCoder.encode(types, values), { gasLimit: 20000000 });
  console.log(result);

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});