const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let LightClientManager = await ethers.getContract('LightClientManager');

    let chainId = 0;
    let contract ="";

    await LightClientManager.register(chainId,contract);
}

module.exports.tags = ['LightClientManagerSet']
