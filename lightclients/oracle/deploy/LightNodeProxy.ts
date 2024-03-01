import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { BigNumber } from "ethers";

let mpt = process.env.MPT_VERIFY || 0;
let chainId = process.env.CHAIN_Id;
let nodeType = process.env.NODE_TYPE;
const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts, ethers } = hre;
    const { deploy } = deployments;

    const { deployer } = await getNamedAccounts();

    console.log("mpt address", mpt);

    if (mpt == undefined || mpt == "") {
        await deploy("MPTVerify", {
            from: deployer,
            args: [],
            log: true,
            contract: "MPTVerify",
        });
        let MPTVerify = await deployments.get("MPTVerify");
        mpt = MPTVerify.address;

        console.log("mpt address", mpt);
    }

    let lightNode = await deployments.get("LightNode");

    let LightNode = await ethers.getContractFactory("LightNode");

    let initData = LightNode.interface.encodeFunctionData("initialize", [chainId, deployer, mpt, nodeType]);

    await deploy("LightNodeProxy", {
        from: deployer,
        args: [lightNode.address, initData],
        log: true,
        contract: "LightNodeProxy",
        //gasLimit: 20000000,
    });
};

export default deploy;
deploy.tags = ["Proxy"];
deploy.dependencies = ["LightNode"];
