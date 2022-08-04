const BigNumber = require('bignumber.js')
const {address} = require("hardhat/internal/core/config/config-validation");
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer,wcoin,mapcoin} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('FeeCenter', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'FeeCenter',
    })

    await deploy('MAPCrossChainServiceRelay', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPCrossChainServiceRelay',
    })

    await deploy('TokenRegister', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'TokenRegister',
    })

    let mcssRelay = await ethers.getContract('MAPCrossChainServiceRelay');
    let feeCenter = await ethers.getContract('FeeCenter');
    let tokenRegister = await ethers.getContract('TokenRegister');

    console.log("MAPCrossChainServiceRelay address:",mcssRelay.address);
    console.log("feeCenter address:",feeCenter.address);
    console.log("tokenRegister address:",tokenRegister.address);

    console.log(wcoin.address);
    console.log(mapcoin.address);

    await mcssRelay.initialize(wcoin.address,mapcoin.address,"0x0000000000747856657269667941646472657373");


}

module.exports.tags = ['MAPCrossChainServiceRelay']