const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());


    await deploy('LightClientManager', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightClientManager',
    })

    let MaintainerManager = await ethers.getContract('LightClientManager');

    console.log("LightClientManager",MaintainerManager.address)
}

module.exports.tags = ['LightClientManager']
