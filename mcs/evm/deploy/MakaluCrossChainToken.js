const configData = require("./config/deployConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);


    await deploy('StandardToken', {
        from: deployer.address,
        args: [configData.mccTokenName,configData.mccTokenSymbol],
        log: true,
        contract: 'StandardToken',
    })

    let mccToken = await ethers.getContract('StandardToken');

    console.log("MakaluCrossChainToken address:",mccToken.address);

}

module.exports.tags = ['MakaluCrossChainToken']