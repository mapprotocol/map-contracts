import { BigNumber ,Contract} from "ethers";
import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { ethers } from "hardhat";
import { BlockHeader, getBlock } from "../utils/Util";

let uri = process.env.BSCURI;
let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen;
let chainId = process.env.CHAINID;
let epochNum = 200;

async function main() {
    let [wallet] = await ethers.getSigners();

    console.log("begin ...");

    const MPTVerify = await ethers.getContractFactory("MPTVerify");

    const mPTVerify = await MPTVerify.deploy();

    await mPTVerify.connect(wallet).deployed();

    console.log("mPTVerify Implementation deployed on:", mPTVerify.address);

    const LightNode = await ethers.getContractFactory("LightNode");

    const lightNode = await LightNode.deploy();

    await lightNode.connect(wallet).deployed();

    console.log("lightNode Implementation deployed on:", lightNode.address);

    const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

    const provider = new ethers.providers.JsonRpcProvider(uri);
    let currentBlock = 39539137 //await provider.getBlockNumber();
    let lastEpoch = currentBlock - (currentBlock % epochNum) - epochNum - epochNum - epochNum;
    let lastHeader = await getBlock(lastEpoch, provider);
    let second = await getBlock(lastEpoch - epochNum, provider);
    let initHeaders: Array<BlockHeader> = new Array<BlockHeader>();
    initHeaders.push(second);
    initHeaders.push(lastHeader);
    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        minEpochBlockExtraDataLen,
        wallet.address,
        mPTVerify.address,
        initHeaders,
    ]);
    const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

    await lightNodeProxy.connect(wallet).deployed();

    console.log("lightNode proxy deployed on:", lightNodeProxy.address);
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
}

async function updateHeader(wallet: SignerWithAddress, lightNode: Contract) {
    const provider = new ethers.providers.JsonRpcProvider(uri);

    let last: BigNumber = await lightNode.headerHeight();

    let headers: Array<BlockHeader> = new Array<BlockHeader>();

    for (let i = 0; i < 9; i++) {
        let lastHeader = await getBlock(last.toNumber() + epochNum + i, provider);
        headers.push(lastHeader);
    }

    await (await lightNode.updateBlockHeader(await lightNode.getHeadersBytes(headers))).wait();

    console.log(await lightNode.headerHeight());
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
