import {HardhatRuntimeEnvironment} from 'hardhat/types';
import {DeployFunction} from 'hardhat-deploy/types';


let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen
let chainId = process.env.CHAINID;


const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {

  const {deployments, getNamedAccounts} = hre;
  const {deploy} = deployments;
  const {deployer} = await getNamedAccounts();

  
  let MPTVerify = await deployments.get('MPTVerify');

  await deploy('LightNode', {
    from: deployer,
    args: [],
    log: true,
    contract:'LightNode'
  });

};

export default deploy;
deploy.tags = ['LightNode'];
deploy.dependencies = ['MPTVerify']