
const  initializeData = require('./configMainnet');

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());
    // console.log(initializeData.initData)
    await deploy('VerifyTool', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'VerifyTool',
    })

    await deploy('Verifier', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'Verifier',
    })

    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let verifyTool = await deployments.get('VerifyTool');
    let verifier = await deployments.get('Verifier');
    let LightNode = await deployments.get('LightNode');
    let lightNode = await ethers.getContractAt("LightNode",LightNode.address)

    console.log(lightNode.address)

    let validatorsInfo = initializeData.validatorsInfo;

    let validatorsCount = initializeData.validatorsCount;

    let epoch = initializeData.epoch;

    let epochSize = initializeData.epochSize;


    let data = lightNode.interface.encodeFunctionData("initialize", [validatorsInfo, validatorsCount, epoch, epochSize,verifyTool.address,verifier.address]);
    console.log("initialize success")


    await deploy('LightNodeProxy', {
        from: deployer.address,
        args: [lightNode.address,data],
        log: true,
        contract: 'LightNodeProxy',
    })

    let lightProxyClient = await deployments.get('LightNodeProxy');

    console.log("lightProxyClient Address",lightProxyClient.address)

}

module.exports.tags = ['LightNodeMainnet']
