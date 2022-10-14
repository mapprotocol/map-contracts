
let lightNodeManagerAddress = "0xE0aa8a8b3930a0d6fc45Faa26aeD2d66e12A0d9a"

//let nearLightNode = "0x3DDe290c361800508a38C1251FC4e67fD88B060A";
let nearLightNode = "0xe96f6c6C7fB5117dF05bE91cc1596607B0423c6b";
let ethLightNode = "0x000068656164657273746F726541646472657373";

module.exports = async function ({ethers, deployments}) {
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let lightNodeManager = await ethers.getContractAt('LightClientManager',lightNodeManagerAddress);


    await (await lightNodeManager.connect(deployer).register(34434,ethLightNode)).wait();
    await (await lightNodeManager.connect(deployer).register(1313161555,nearLightNode)).wait();
    console.log("lightNodeManager register success");

}

module.exports.tags = ['LightClientManagerSet']