import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { BigNumber, Contract } from "ethers";
import { ethers } from "hardhat";
import {
  BlockHeader, getBlock,
  getProof
} from "../utils/Util"



let uri: string = process.env.MATICURI || "";
let minEpochBlockExtraDataLen = 161
let epochNum = 64;




async function main() {

  let [wallet] = await ethers.getSigners();

  console.log("begin ...");

  const MPTVerify = await ethers.getContractFactory("MPTVerify");

  const mPTVerify = await MPTVerify.deploy();

  await mPTVerify.connect(wallet).deployed();

  console.log("mPTVerify Implementation deployed on:", mPTVerify.address);

  const LightNode = await ethers.getContractFactory("LightNode");

  const lightNode = await LightNode.deploy(minEpochBlockExtraDataLen, wallet.address, mPTVerify.address);

  await lightNode.connect(wallet).deployed();

  console.log("lightNode Implementation deployed on:", lightNode.address);

  const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

  const provider = new ethers.providers.JsonRpcProvider(uri);

  let currentBlock = await provider.getBlockNumber()

  let lastEpoch = currentBlock - currentBlock % epochNum - 1 - epochNum;

  let lastHeader = await getBlock(34765823, provider);

  console.log("init == ",lastHeader);

  let initData = LightNode.interface.encodeFunctionData("initialize", [minEpochBlockExtraDataLen, wallet.address, mPTVerify.address, lastHeader]);

  const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

  await lightNodeProxy.connect(wallet).deployed();

  console.log("lightNode proxy deployed on:", lightNodeProxy.address);

  await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
  
  await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

  let txHash = '0xbf684bda3767bd3b756e03f441c1b36b68c09ef5795702af642eddf884053e29';

  await verify(txHash,uri,LightNode.attach(lightNodeProxy.address));
}


async function updateHeader(wallet: SignerWithAddress, lightNode: Contract) {

  const provider = new ethers.providers.JsonRpcProvider(uri);

  let last: BigNumber = await lightNode.headerHeight();

  // console.log(last);

  let lastHeader = await getBlock(last.toNumber() + epochNum, provider);

  console.log("addBlock == ",lastHeader);

  await (await lightNode.updateBlockHeader(await lightNode.getHeadersBytes(lastHeader))).wait();

  console.log(await lightNode.headerHeight());
}


async function verify(txHash: string, rpc: string, lightNode: Contract) {

  let proof = await getProof(txHash, rpc);

  console.log(proof);

  let result = await lightNode.verifyProofData(await lightNode.getBytes(proof));

  console.log(result);

}



// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
