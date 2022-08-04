const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer,wcoin,mapcoin} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let lightNodeAddress = "0x28EF57E733C6F67067078e14B3CE159543a8A3c8";


    await deploy('MapCrossChainService', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MapCrossChainService',
    })

    let mcss = await ethers.getContract('MapCrossChainService');

    console.log("MapCrossChainService address:",mcss.address);


    await mcss.initialize(wcoin.address,mapcoin.address,lightNodeAddress);
    console.log("MapCrossChainService initialize success");

}

module.exports.tags = ['MapCrossChainService']