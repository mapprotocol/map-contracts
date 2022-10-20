const configData = require("./config/tokenCrossChainConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',configData.mcsAddress);

    if (configData.nearChainId !== ""){
        await (await mcssProxy.connect(deployer).setChainId(
            configData.nearChainId
        )).wait();

        await (await mcssProxy.connect(deployer).setCanBridgeToken(
            configData.mcsMotTokenAddress,
            configData.relayChainId,
            true
        )).wait();
        await (await mcssProxy.connect(deployer).setCanBridgeToken(
            configData.mcsMotTokenAddress,
            configData.nearChainId,
            true
        )).wait();

        console.log("MapCrossChainService set map and near setCanBridgeToken success");

    }else {
        await (await mcssProxy.connect(deployer).setCanBridgeToken(
            configData.mcsMotTokenAddress,
            configData.relayChainId,
            true
        )).wait();

        console.log("MapCrossChainService setCanBridgeToken success");
    }




}

module.exports.tags = ['MapCrossChainServiceSetToken']