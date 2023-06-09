import { DeployFunction } from 'hardhat-deploy/types';

const deploy: DeployFunction = async function (hre:any) {

  const { deployments, getNamedAccounts ,ethers} = hre;
  const { deploy } = deployments;
  const { deployer } = await getNamedAccounts();

  await deploy('LightNode', {
    from: deployer,
    args: [],
    log: true,
    contract: 'LightNode'
  });

  let lightNode = await ethers.getContract('LightNode');

  console.log("BTC LightNode success：",lightNode.address)

};

export default deploy;
deploy.tags = ['LightNode'];
