import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { BigNumber, Contract } from "ethers";
import { ethers } from "hardhat";
import { BlockHeader, getBlock, getProof } from "../utils/Util";

let uri: string = process.env.RPCURI || "";
let minEpochBlockExtraDataLen = 417;
let chainId = process.env.CHAINID;
let confirms = 5; //process.env.CONFIRMS || 10
let epochNum = 64;

async function main() {
    let [wallet] = await ethers.getSigners();
    // await test()

    let proof = await getProof(
        "0x1d66c15242accdb643671160734ef2af9eb79b2ea88cd9cad9a06329098525a1",
        "https://rpc.bittorrentchain.io",
        5
    );

    console.log(proof);
}

async function test() {
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

    const provider = new ethers.providers.JsonRpcProvider("https://rpc.bittorrentchain.io");

    let currentBlock = 26326655; //await provider.getBlockNumber()

    let lastEpoch = currentBlock - (currentBlock % epochNum) - 1 - epochNum;

    let lastHeader = await getBlock(lastEpoch, provider);

    console.log("init == ", lastHeader);

    console.log(chainId);

    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        minEpochBlockExtraDataLen,
        wallet.address,
        mPTVerify.address,
        confirms,
        lastHeader,
    ]);

    const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

    await lightNodeProxy.connect(wallet).deployed();

    console.log("lightNode proxy deployed on:", lightNodeProxy.address);

    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

    await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

    // await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

    // await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

    // await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));
}

async function updateHeader(wallet: SignerWithAddress, lightNode: Contract) {
    const provider = new ethers.providers.JsonRpcProvider(uri);

    let last: BigNumber = await lightNode.headerHeight();

    console.log(last);

    let headers: Array<BlockHeader> = new Array<BlockHeader>();

    let epoch = epochNum;

    for (let i = 0; i < confirms; i++) {
        let lastHeader = await getBlock(last.toNumber() + epoch + i, provider);
        lastHeader.miner = "0x0000000000000000000000000000000000000000";
        headers.push(lastHeader);
    }

    console.log(headers);

    let bytes = await lightNode.getHeadersBytes(headers);

    // console.log();

    await (await lightNode.updateBlockHeader(bytes, { gasLimit: 8000000 })).wait();

    console.log(await lightNode.headerHeight());
}

async function verify(txHash: string, rpc: string, lightNode: Contract) {
    let proof = await getProof(txHash, rpc, confirms);

    console.log(proof);

    let result = await lightNode.verifyProofData(await lightNode.getBytes(proof));

    console.log(result);
    //29563862   29551103
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
