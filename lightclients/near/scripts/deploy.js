const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('../test/utils/borsh');
const hre = require("hardhat");
const { promisify } = require('util');
const { SourceLocation } = require('hardhat/internal/hardhat-network/stack-traces/model');
const sleep = promisify(setTimeout);
const { Web3 } = require('../test/utils/robust')



async function main() {

  let [wallet] = await hre.ethers.getSigners();
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

  await (await proxy.updateBlockHeader(borshify(require('./data/addBlock.json')), { gasLimit: 20000000 })).wait();

  console.log(await proxy.headerHeight());

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
