
const configData = require("./config/initiateConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('MAPCrossChainServiceRelay', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPCrossChainServiceRelay',
    })

    let mcssRelay = await ethers.getContract('MAPCrossChainServiceRelay');

    console.log("MAPCrossChainServiceRelay up address:",mcssRelay.address);

    let mcssRelayP = await ethers.getContractAt('MAPCrossChainServiceRelay',configData.relayAddress);

    await (await mcssRelayP.upgradeTo(mcssRelay.address)).wait();

    console.log("MAPCrossChainServiceRelay up success");

}

module.exports.tags = ['MAPCrossChainServiceRelayProxyUp']