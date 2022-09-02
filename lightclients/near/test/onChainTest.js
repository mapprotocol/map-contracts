const hre = require("hardhat");
const { ethers } = require("hardhat");
const { promisify } = require('util');
const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('./utils/borsh');
const sleep = promisify(setTimeout);


async function main() {

  await verifyProofData();

  // await test();

}



async function verifyProofData() {
  let [wallet] = await ethers.getSigners();

  const LightNode = await hre.ethers.getContractFactory("LightNode");
  const lightNode = await LightNode.connect(wallet).deploy();
  await lightNode.deployed();
  console.log("Implementation deployed to .....", lightNode.address);

  const LightNodeProxy = await hre.ethers.getContractFactory("LightNodeProxy");

  const iface = new hre.ethers.utils.Interface([
    "function initialize(address _owner, bytes[2] memory initDatas)"

  ]);

  let block = '0x' + borshify(require('./data/block.json')).toString('hex');
  let validators = '0x' + borshifyInitialValidators(require('./data/validators.json').next_bps).toString('hex');
  let arr = [validators, block];
  let data = iface.encodeFunctionData("initialize", [wallet.address, arr]);
  const lightNodeProxy = await LightNodeProxy.connect(wallet).deploy(lightNode.address, data);
  await lightNodeProxy.deployed();

  console.log("lightNodeProxy deployed to .....", lightNodeProxy.address);

  const proxy = LightNode.attach(lightNodeProxy.address);

  console.log(await proxy.headerHeight());
  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock.json')), { gasLimit: 20000000 })).wait()

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock1.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock2.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock3.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock4.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock5.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock6.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock7.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock8.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock9.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock10.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock11.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock12.json')), { gasLimit: 20000000 })).wait();

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


  let proof3 = "0x" + borshifyOutcomeProof(require('./data/proof3.json')).toString('hex');

  let head1 = "0x" + borshify(require('./data/addBlock11.json')).toString('hex');

  values = [
    head1,
    proof3
  ]

  await sleep(30000);
  result = await proxy.verifyProofData(ethers.utils.defaultAbiCoder.encode(types, values), { gasLimit: 20000000 });
  console.log(result);

}


async function test() {

  const LightNode = await hre.ethers.getContractFactory("LightNode");
  let proxy = LightNode.attach("0x3CE319B86ad4CC0623F7039C48978c1A2c6cF8eB");

  // console.log(await proxy.headerHeight());

  // await (await proxy.updateBlockHeader(borshify(require('./data/addBlock13.json')), { gasLimit: 20000000 })).wait();

  // console.log(await proxy.headerHeight());


  // console.log(await proxy.headerHeight());

  // let head = "0x" + borshify(require('./data/addBlock27.json')).toString('hex');
  // let proof = "0x" + borshifyOutcomeProof(require('./data/proof4.json')).toString('hex');

  // let types = [
  //   'bytes',
  //   'bytes'
  // ]

  // let values = [
  //   head,
  //   proof
  // ]


  // let result = await proxy.verifyProofData(ethers.utils.defaultAbiCoder.encode(types, values), { gasLimit: 20000000 });

  // console.log(result);

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});