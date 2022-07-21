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


    await deploy('MaintainerManager', {
        from: deployer.address,
        args: ["0","10000000000"],
        log: true,
        contract: 'MaintainerManager',
    })

    let MaintainerManager = await ethers.getContract('MaintainerManager');

    console.log(MaintainerManager.address)



}

module.exports.tags = ['MaintainerManager']
