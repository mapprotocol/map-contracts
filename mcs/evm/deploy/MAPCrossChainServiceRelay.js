const BigNumber = require('bignumber.js')
const {address} = require("hardhat/internal/core/config/config-validation");
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

const configData = require("./config/deployConfig.js");

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

    console.log("MAPCrossChainServiceRelay address:",mcssRelay.address);

    let data = await mcssRelay.initialize(configData.relayWmapAddress,configData.relayMapTokenAddress,configData.relayLightNodeManagerAddress);
    console.log("init success");

    await deploy('MAPCrossChainServiceRelayProxy', {
        from: deployer.address,
        args: [mcssRelay.address,data.data],
        log: true,
        contract: 'MAPCrossChainServiceRelayProxy',
    })

    let mcssRelayP = await ethers.getContract('MAPCrossChainServiceRelayProxy');

    console.log("MAPCrossChainServiceRelayProxy address:",mcssRelayP.address);

}

module.exports.tags = ['MAPCrossChainServiceRelay']