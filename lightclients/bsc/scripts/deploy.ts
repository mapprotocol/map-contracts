import { BigNumber } from "ethers";
import { ethers } from "hardhat";
const { GetProof } = require('eth-proof')
import {
  BlockHeader, getBlock,
  TxLog, ReceiptProof,
  TxReceipt, index2key,
  ProofData, DProofData
} from "../utils/Util"
import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
const { encode } = require('eth-util-lite')
// const { Keccak } = require('sha3')
// const Rpc = require('isomorphic-rpc')
// import { JsonRpcProvider } from "@ethersproject/providers";


let uri = process.env.BSCURI;
let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen
let chainId = process.env.CHAINID;
let epochNum = 200;
let proofTx = '0xcc47eac7b22ab71494a4d605aedda7c8e39a949ae82dc6a14bc6358fee67ad56';



async function main() {

  let [wallet] = await ethers.getSigners();
  await deployLightNode(wallet);
}

function getValidators(extraData: string) {
  let length = extraData.length;
  return extraData.substring(66, length - 130);
}


async function getReceipt(txHash: string, uri?: string) {

  let p = new GetProof(uri);

  const resp = await p.receiptProof(txHash)

  let proofs: Array<string> = new Array<string>();

  for (let i = 0; i < resp.receiptProof.length; i++) {

    proofs[i] = '0x' + encode(resp.receiptProof[i]).toString('hex');
  }

  return {
    // root: resp.header.receiptRoot.toString('hex'),
    proof: proofs,
    key: '0x' + encode(Number(resp.txIndex)).toString('hex') // '0x12' => Nunmber
  }


}


async function deployLightNode(wallet: SignerWithAddress) {

  const LightNode = await ethers.getContractFactory("LightNode");

  const lightNode = await LightNode.deploy(chainId, minEpochBlockExtraDataLen, wallet.address);

  await lightNode.connect(wallet).deployed();

  console.log("lightNode Implementation deployed on:", lightNode.address);

  const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

  const provider = new ethers.providers.JsonRpcProvider(uri);

  let currentBlock = 20853200 //await provider.getBlockNumber()

  let lastEpoch = currentBlock - currentBlock % epochNum - epochNum;

  let lastHeader = await getBlock(lastEpoch, provider);

  let second = await getBlock(lastEpoch - epochNum, provider);

  let initHeaders: Array<BlockHeader> = new Array<BlockHeader>();

  initHeaders.push(second);

  initHeaders.push(lastHeader);

  let initData = LightNode.interface.encodeFunctionData("initialize", [chainId, minEpochBlockExtraDataLen, wallet.address, initHeaders]);

  const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

  await lightNodeProxy.connect(wallet).deployed();

  console.log("lightNode proxy deployed on:", lightNodeProxy.address);

  let proxy = LightNode.attach(lightNodeProxy.address);


  let current = BigNumber.from(await proxy.headerHeight()).toNumber();

  console.log(current);

  let addHearders: Array<BlockHeader> = new Array<BlockHeader>();

  let preValidators = '0x' + getValidators(lastHeader.extraData);

  let max = Math.floor((preValidators.length - 2) / 80) + 1

  for (let i = 0; i < max; i++) {

    let t = await getBlock(current + epochNum + i, provider);

    addHearders.push(t);

    console.log(t.number);
  }

  let re = await (await proxy.updateBlockHeader(await lightNode.getHeadersBytes(addHearders))).wait();

  console.log(re);

  current = BigNumber.from(await proxy.headerHeight()).toNumber();

  console.log(current);

  let r = await provider.getTransactionReceipt(proofTx);

  let proofHearders: Array<BlockHeader> = new Array<BlockHeader>();

  for (let i = 0; i < max; i++) {

    let t = await getBlock(r.blockNumber + i, provider);

    proofHearders.push(t);
  }


  let logs: TxLog[] = new Array<TxLog>();

  for (let i = 0; i < r.logs.length; i++) {

    let log = new TxLog(r.logs[i].address, r.logs[i].topics, r.logs[i].data);

    logs.push(log);
  }

  let txReceipt = new TxReceipt(BigNumber.from(r.type), BigNumber.from(r.status || r.root).toHexString(), BigNumber.from(r.cumulativeGasUsed), r.logsBloom, logs);

  let proof = await getReceipt(proofTx, uri);

  let receiptProof = new ReceiptProof(txReceipt, index2key(BigNumber.from(r.transactionIndex).toNumber(), proof.proof.length), proof.proof);

  let proofData = new DProofData(proofHearders, receiptProof);

  let result = await proxy.verifyProofData(await proxy.getBytes(proofData), { gasLimit: 20000000 });

  console.log("result ==", result);


  // const TestVerify = await ethers.getContractFactory("TestVerify");

  // const testVerify = await TestVerify.deploy();

  // await testVerify.connect(wallet).deployed();

  // let g = await (await testVerify.verify(proxy.address,await proxy.getBytes(proofData),{gasLimit: 20000000 })).wait();

  // console.log(g);

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
