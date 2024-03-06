import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { BigNumber, Contract } from "ethers";
import { ethers } from "hardhat";
import { getProof } from "../utils/Util";

let chainId = process.env.CHAIN_Id;
let uri = "";

async function main() {
    let [wallet] = await ethers.getSigners();

    console.log("begin ...");

    // const MPTVerify = await ethers.getContractFactory("MPTVerify");

    // const mPTVerify = await MPTVerify.deploy();

    // await mPTVerify.connect(wallet).deployed();

    // console.log("mPTVerify Implementation deployed on:", mPTVerify.address);

    // const LightNode = await ethers.getContractFactory("LightNode");

    // const lightNode = await LightNode.deploy();

    // await lightNode.connect(wallet).deployed();

    // console.log("lightNode Implementation deployed on:", lightNode.address);

    // const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

    // let initData = LightNode.interface.encodeFunctionData("initialize", [chainId, wallet.address, mPTVerify.address]);

    // const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

    // await lightNodeProxy.connect(wallet).deployed();

    // console.log("lightNode proxy deployed on:", lightNodeProxy.address);

    // await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

    // await updateHeader(wallet, LightNode.attach(lightNodeProxy.address));

    // let txHash = "";

    // await verify(txHash, uri, LightNode.attach(lightNodeProxy.address));
    let rpc = "https://sepolia-rollup.arbitrum.io/rpc"
    let txHash = "0x26193897b8d0a26f2b12669010954eb037ba33c164f1e921db9616611e438bb0";
    const LightNode = await ethers.getContractFactory("LightNode");
    let lightNode = LightNode.attach("0x3eD93c7c36Cf2ceE748eaE3B268F76a766cb8f48");

    await verify(txHash,rpc,lightNode)
}

async function updateHeader(wallet: SignerWithAddress, lightNode: Contract) {}

async function verify(txHash: string, rpc: string, lightNode: Contract) {
    let proof = await getProof(txHash, rpc);

    console.log(proof);

    let result = await lightNode.verifyProofData(await lightNode.getBytes(proof));

    console.log(result);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
