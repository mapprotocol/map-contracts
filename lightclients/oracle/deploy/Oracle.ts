import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts } = hre;
    const { deploy } = deployments;
    const { deployer } = await getNamedAccounts();

    await deploy("Oracle", {
        from: deployer,
        args: [deployer],
        log: true,
        contract: "Oracle",
    });
};

export default deploy;
deploy.tags = ["Oracle"];
