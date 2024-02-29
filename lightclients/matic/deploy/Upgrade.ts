import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts, ethers } = hre;
    const { deploy } = deployments;

    const { deployer } = await getNamedAccounts();

    let LightNodeDeploy = await deploy("LightNode", {
        from: deployer,
        args: [],
        log: true,
        contract: "LightNode",
    });

    let LightNodeProxy = await deployments.get("LightNodeProxy");

    const LightNode = await ethers.getContractFactory("LightNode");

    let proxy = LightNode.attach(LightNodeProxy.address);

    // console.log("mpt before: ", await proxy.mptVerify());

    console.log("implementation before: ", await proxy.getImplementation());

    await (await proxy.upgradeTo(LightNodeDeploy.address)).wait();

    console.log("implementation after: ", await proxy.getImplementation());

    // await (await proxy.setMptVerify("0x4b1EE84A72b44B78346e069D1c66509940827E22")).wait();

    // console.log("mpt after: ", await proxy.mptVerify());
};

export default deploy;
deploy.tags = ["Upgrade"];
