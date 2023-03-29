import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';
import { BigNumber, ethers } from 'ethers';
import {
  req
} from "../utils/httpUtil"
import {
  BlockHeader,
} from "../utils/Util"

let uri = process.env.RPC_URI || "";
let mpt = process.env.MPT_VERIFY || 0;
let start = process.env.START_SYNCY_BLOCK
let chainId = process.env.CHAIN_Id
let epochNum = 430;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, ethers } = hre;
  const { deploy } = deployments;

  const { deployer } = await getNamedAccounts();

  if (mpt == undefined || mpt == '') {
    await deploy('MPTVerify', {
      from: deployer,
      args: [],
      log: true,
      contract: 'MPTVerify'
    });
    let MPTVerify = await deployments.get('MPTVerify');
    mpt = MPTVerify.address;
  }

  let lightNode = await deployments.get('LightNode');

  let currentBlock: number = BigNumber.from(start).toNumber();

  if (currentBlock == undefined || currentBlock == 0) {
    currentBlock = await getLastBlockNumber(uri);
  }

  let lastEpoch = currentBlock - currentBlock % epochNum - epochNum;;

  let lastHeader = await getBlockHeader(uri, lastEpoch);

  let validators = await getValidators(uri, lastEpoch + 1)

  let LightNode = await ethers.getContractFactory("LightNode")

  let initData = LightNode.interface.encodeFunctionData("initialize", [chainId, deployer, mpt, lastHeader, validators]);

  await deploy('LightNodeProxy', {
    from: deployer,
    args: [lightNode.address, initData],
    log: true,
    contract: 'LightNodeProxy',
    gasLimit: 12000000
  });

};

async function getLastBlockNumber(uri: string) {

  let methons = "platon_blockNumber";

  let params: any[] = [];

  let data = await req(uri, methons, params);

  return BigNumber.from(data.result).toNumber();

}


async function getBlockHeader(uri: string, blockNumber: number) {

  let methons = "eth_getBlockByNumber";

  let params: any[] = [ethers.utils.hexStripZeros(BigNumber.from(blockNumber).toHexString()), false];
  let data = await req(uri, methons, params);
  let rpcHeader = data.result;
  let blockHeader = new BlockHeader(rpcHeader.parentHash,
    rpcHeader.miner, rpcHeader.stateRoot, rpcHeader.transactionsRoot,
    rpcHeader.receiptsRoot, rpcHeader.logsBloom, BigNumber.from(rpcHeader.number),
    BigNumber.from(rpcHeader.gasLimit), BigNumber.from(rpcHeader.gasUsed), BigNumber.from(rpcHeader.timestamp),
    rpcHeader.extraData, rpcHeader.nonce);

  return blockHeader;
}

async function getValidators(uri: string, blockNumber: number) {
  let methons = "debug_getValidatorByBlockNumber";
  let params: any[] = [blockNumber];
  let data = await req(uri, methons, params);
  let v: any[] = JSON.parse(data.result)
  v.map((e) => {
    e.BlsPubKey = "0x" + e.BlsPubKey;
    e.NodeId = "0x" + e.NodeId
  });

  return v;
}



export default deploy;
deploy.tags = ['Proxy'];
deploy.dependencies = ['LightNode']