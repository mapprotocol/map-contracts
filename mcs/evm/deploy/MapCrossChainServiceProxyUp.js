const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

const configData = require("./config/initiateConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('MapCrossChainService', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MapCrossChainService',
    })

    let mcss = await ethers.getContract('MapCrossChainService');


    console.log("MapCrossChainService up address:",mcss.address);


    let mcssProxy = await ethers.getContractAt('MapCrossChainService',configData.mcsAddress);

    await (await mcssProxy.upgradeTo(mcss.address)).wait();

    console.log("MapCrossChainService up success")

}

module.exports.tags = ['MapCrossChainServiceProxyUp']