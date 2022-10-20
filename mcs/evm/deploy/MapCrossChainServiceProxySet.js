const configData = require("./config/initiateConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',configData.mcsAddress);


    await (await mcssProxy.connect(deployer).setBridge(configData.relayAddress,configData.relayChainId)).wait();

    console.log("MapCrossChainService set success");

}

module.exports.tags = ['MapCrossChainServiceProxySet']