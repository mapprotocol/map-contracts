import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';


let minEpochBlockExtraDataLen = 161

let mpt = process.env.MPT_VERIFY;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {

  const { deployments, getNamedAccounts } = hre;
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


  await deploy('LightNode', {
    from: deployer,
    args: [minEpochBlockExtraDataLen, deployer, mpt],
    log: true,
    contract: 'LightNode'
  });

};

export default deploy;
deploy.tags = ['LightNode'];
// deploy.dependencies = ['MPTVerify']