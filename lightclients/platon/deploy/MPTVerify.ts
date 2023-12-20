import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts } = hre;
    const { deploy } = deployments;
    const { deployer } = await getNamedAccounts();

    await deploy("MPTVerify", {
        from: deployer,
        args: [],
        log: true,
        contract: "MPTVerify",
    });
};

export default deploy;
deploy.tags = ["MPTVerify"];
