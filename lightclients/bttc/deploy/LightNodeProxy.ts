import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { BigNumber } from "ethers";
import { BlockHeader, getBlock } from "../utils/Util";

let uri = process.env.RPCURI;
let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen;
let mpt = process.env.MPT_VERIFY || 0;
let start = process.env.START_SYNCY_BLOCK;
let chainId = process.env.CHAINID;
let confirms = process.env.CONFIRMS;
let epochNum = 64;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts, ethers } = hre;
    const { deploy } = deployments;

    const { deployer } = await getNamedAccounts();

    if (mpt == undefined || mpt == "") {
        await deploy("MPTVerify", {
            from: deployer,
            args: [],
            log: true,
            contract: "MPTVerify",
        });
        let MPTVerify = await deployments.get("MPTVerify");
        mpt = MPTVerify.address;
    }

    let lightNode = await deployments.get("LightNode");

    const provider = new ethers.providers.JsonRpcProvider(uri);

    let currentBlock: number = BigNumber.from(start).toNumber();

    if (currentBlock == undefined || currentBlock == 0) {
        currentBlock = await provider.getBlockNumber();
    }

    let lastEpoch = currentBlock - (currentBlock % epochNum) - 1 - epochNum;

    let lastHeader = await getBlock(lastEpoch, provider);

    console.log(lastHeader);

    let LightNode = await ethers.getContractFactory("LightNode");

    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        minEpochBlockExtraDataLen,
        deployer,
        mpt,
        confirms,
        lastHeader,
    ]);

    await deploy("LightNodeProxy", {
        from: deployer,
        args: [lightNode.address, initData],
        log: true,
        contract: "LightNodeProxy",
        gasLimit: 10000000,
    });
};

export default deploy;
deploy.tags = ["Proxy"];
deploy.dependencies = ["LightNode"];
