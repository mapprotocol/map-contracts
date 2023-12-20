import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts, ethers } = hre;
    const { deploy } = deployments;
    const { deployer } = await getNamedAccounts();

    /*
  let mptVerify = await deploy('MPTVerify', {
    from: deployer,
    args: [],
    log: true,
    contract: 'MPTVerify'
  });

  let mpt_address = mptVerify.address;
  */
    let mpt_address = "0x4b1ee84a72b44b78346e069d1c66509940827e22";

    let LightNodeProxy = await deployments.get("LightNodeProxy");

    console.log("light node proxy address is :", LightNodeProxy.address);

    const LightNode = await ethers.getContractFactory("LightNode");

    let proxy = LightNode.attach(LightNodeProxy.address);

    let old_verify = await proxy.mptVerify();

    console.log("old mptVerify address is :", old_verify);

    await proxy.setMptVerify(mpt_address);

    let new_verify = await proxy.mptVerify();

    console.log("new mptVerify address is :", new_verify);
};

export default deploy;
deploy.tags = ["MPTVerify"];
