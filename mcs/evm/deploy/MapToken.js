const configData = require("./config/deployConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);


    await deploy('StandardToken', {
        from: deployer.address,
        args: [configData.mapTokenName,configData.mapTokenSymbol],
        log: true,
        contract: 'StandardToken',
    })

    let mapToken = await ethers.getContract('StandardToken');

    console.log("MapToken address:",mapToken.address);

}

module.exports.tags = ['MapToken']