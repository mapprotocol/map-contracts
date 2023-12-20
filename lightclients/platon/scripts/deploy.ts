import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { BigNumber, Contract } from "ethers";
import { ethers } from "hardhat";
let data = require("../data/blocks.js");

let chainId = process.env.CHAIN_Id;

async function main() {
    await upgrandle();
}

async function upgrandle() {
    let [wallet] = await ethers.getSigners();
    const LightNode = await ethers.getContractFactory("LightNode");
    const lightNode = await LightNode.deploy();

    await lightNode.connect(wallet).deployed();

    let proxy = LightNode.attach("0xf15A676394531D869afeC1a94DEa9b0287c3b077");

    console.log(await proxy.headerHeight());

    // console.log('implementation before: ', await proxy.getImplementation());

    // await (await proxy.upgradeTo(lightNode.address)).wait();

    // console.log('implementation after: ', await proxy.getImplementation());
}

async function updateHeader(wallet: SignerWithAddress, lightNode: Contract) {}

async function deploy() {
    let [wallet] = await ethers.getSigners();
    const MPTVerify = await ethers.getContractFactory("MPTVerify");

    const mPTVerify = await MPTVerify.deploy();

    await mPTVerify.connect(wallet).deployed();

    const LightNode = await ethers.getContractFactory("LightNode");
    const lightNode = await LightNode.deploy();

    await lightNode.connect(wallet).deployed();

    const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

    let initBlock = data.initBlock;

    let validators = data.init_validators;

    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        wallet.address,
        mPTVerify.address,
        initBlock,
        validators,
    ]);

    const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);

    await lightNodeProxy.connect(wallet).deployed();

    let proxy = LightNode.attach(lightNodeProxy.address);

    console.log("proxy", proxy.address);

    await (
        await proxy.updateBlockHeader(
            await lightNode.getHeadersBytes(data.addBlock1, data.quorumCert1, data.validators1)
        )
    ).wait();

    let current = await proxy.headerHeight();

    console.log(current);
    await (
        await proxy.updateBlockHeader(
            await lightNode.getHeadersBytes(data.addBlock2, data.quorumCert2, data.validators2)
        )
    ).wait();

    current = await proxy.headerHeight();
    console.log(current);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
