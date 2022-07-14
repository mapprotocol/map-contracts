// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// You can also run a script with `npx hardhat run <script>`. If you do that, Hardhat
// will compile your contracts, add the Hardhat Runtime Environment's members to the
// global scope, and execute the script.
const hre = require("hardhat");


let nearProofProducerAccount_ = "0x6175726f7261";

async function main() {

  let [wallet] = await hre.ethers.getSigners();

  const LightNode = await hre.ethers.getContractFactory("LightNode");
  const lightNode = await LightNode.connect(wallet).deploy();
  await lightNode.deployed();
  console.log("LightNode deployed to .....", lightNode.address);
  await lightNode.initialize(nearProofProducerAccount_);

  // await lightNode.connect(wallet).initWithValidators(validators, { gasLimit: 20000000 });
  // await lightNode.connect(wallet).initWithBlock(block, { gasLimit: 20000000 });
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
