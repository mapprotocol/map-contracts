import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { BlockHeader, getBlock } from "../utils/Util";
import { BigNumber } from "ethers";

let uri = process.env.BSCURI;
let chainId = process.env.CHAINID;
let start = process.env.START_SYNCY_BLOCK || 0;
let epochNum = 200;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const { deployments, getNamedAccounts, ethers } = hre;
    const { deploy } = deployments;

    const { deployer } = await getNamedAccounts();

    // let mPTVerify = await deployments.get("MPTVerify");

    let mpt_addr = "0x81D26E2387059CF43ADA1c11c12D5d6627184fA1";

    let lightNode = await deployments.get("LightNodeV2");

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

    let LightNode = await ethers.getContractFactory("LightNodeV2");

    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        deployer,
        mpt_addr,
        initHeaders,
    ]);

    await deploy("LightNodeProxyV2", {
        from: deployer,
        args: [lightNode.address, initData],
        log: true,
        contract: "LightNodeProxy",
        gasLimit: 10000000,
    });
};

export default deploy;
deploy.tags = ["ProxyV2"];
deploy.dependencies = ["LightNodeV2"];
