const configData = require("./config/initiateConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',configData.relayAddress);

    await (await mcssRelayProxy.connect(deployer).setIdTable(configData.mcsNearChainId,1)).wait();

    await (await mcssRelayProxy.connect(deployer).setChainId(configData.mcsNearChainId)).wait();

    await (await mcssRelayProxy.connect(deployer).setBridageAddress(configData.mcsNearChainId,configData.nearExecuteId)).wait();

    console.log("mcssRelay set near cross success");


}

module.exports.tags = ['MAPCrossChainServiceRelayProxyNearSet']