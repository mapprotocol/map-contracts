import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';



let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen
let chainId = process.env.CHAINID;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, ethers } = hre;
  const { deploy } = deployments;

  const { deployer } = await getNamedAccounts();



  let LightNodeDeploy = await deploy('LightNode', {
    from: deployer,
    args: [],
    log: true,
    contract: 'LightNode'
  });

  let LightNodeProxy = await deployments.get('LightNodeProxy');

  const LightNode = await ethers.getContractFactory("LightNode");

  let proxy = LightNode.attach(LightNodeProxy.address);

  console.log('implementation before: ', await proxy.getImplementation());

  await (await proxy.upgradeTo(LightNodeDeploy.address)).wait();

  console.log('implementation after: ', await proxy.getImplementation());

};

export default deploy;
deploy.tags = ['Upgrade'];
