import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';
import {
  BlockHeader, getBlock
} from "../utils/Util"


let uri = process.env.MATICURI;
let minEpochBlockExtraDataLen = 161
let mpt = process.env.MPT_VERIFY;
let epochNum = 64;

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

  const provider = new ethers.providers.JsonRpcProvider(uri);

  let currentBlock = await provider.getBlockNumber()

  let lastEpoch = currentBlock - currentBlock % epochNum - 1 - epochNum;;

  let lastHeader = await getBlock(lastEpoch, provider);

  console.log(lastHeader);

  let LightNode = await ethers.getContractFactory("LightNode")

  let initData = LightNode.interface.encodeFunctionData("initialize", [minEpochBlockExtraDataLen, deployer, mpt, lastHeader]);

  await deploy('LightNodeProxy', {
    from: deployer,
    args: [lightNode.address, initData],
    log: true,
    contract: 'LightNodeProxy',
    gasLimit: 20000000
  });

};

export default deploy;
deploy.tags = ['Proxy'];
deploy.dependencies = ['LightNode']