import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { BlockHeader, getBlock } from "../utils/Util";
import { BigNumber } from "ethers";

let uri = process.env.BSCURI;
let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen;
let chainId = process.env.CHAINID;
let start = process.env.START_SYNCY_BLOCK || 0;
let epochNum = 200;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts, ethers } = hre;
    const { deploy } = deployments;

    const { deployer } = await getNamedAccounts();

    let mPTVerify = await deployments.get("MPTVerify");

    let lightNode = await deployments.get("LightNode");

    const provider = new ethers.providers.JsonRpcProvider(uri);

    let currentBlock: number = BigNumber.from(start).toNumber();

    if (currentBlock == undefined || currentBlock == 0) {
        currentBlock = await provider.getBlockNumber();
    }
    let lastEpoch = currentBlock - (currentBlock % epochNum) - epochNum;

    let lastHeader = await getBlock(lastEpoch, provider);

    console.log(lastHeader);

    let second = await getBlock(lastEpoch - epochNum, provider);

    let initHeaders: Array<BlockHeader> = new Array<BlockHeader>();

    initHeaders.push(second);

    initHeaders.push(lastHeader);

    let LightNode = await ethers.getContractFactory("LightNode");

    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        minEpochBlockExtraDataLen,
        deployer,
        mPTVerify.address,
        initHeaders,
    ]);

    await deploy("LightNodeProxy", {
        from: deployer,
        args: [lightNode.address, initData],
        log: true,
        contract: "LightNodeProxy",
        gasLimit: 20000000,
    });
};

export default deploy;
deploy.tags = ["Proxy"];
deploy.dependencies = ["LightNode"];
