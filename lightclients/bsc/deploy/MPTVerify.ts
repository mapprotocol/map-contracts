import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {

  const { deployments, getNamedAccounts,ethers} = hre;
  const { deploy } = deployments;
  const { deployer } = await getNamedAccounts();

  let mptVerify = await deploy('MPTVerify', {
    from: deployer,
    args: [],
    log: true,
    contract: 'MPTVerify'
  });

  let LightNodeProxy = await deployments.get('LightNodeProxy');

  const LightNode = await ethers.getContractFactory("LightNode");

  let proxy = LightNode.attach(LightNodeProxy.address);

  let old_verify = await proxy.mptVerify();

  console.log("old mptVerify address is :",old_verify);

  await proxy.setMptVerify(mptVerify.address);

  let new_verify = await proxy.mptVerify();

  console.log("new mptVerify address is :",new_verify);

};

export default deploy;
deploy.tags = ['MPTVerify'];