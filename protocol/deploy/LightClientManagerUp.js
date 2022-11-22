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


    await deploy('LightClientManager', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightClientManager',
    })

    let LightClientManager = await ethers.getContract('LightClientManager');

    console.log("LightClientManager",LightClientManager.address);

    let LightClientManagerProxy = await ethers.getContract('LightClientManagerProxy');

    console.log("LightClientManagerProxy",LightClientManagerProxy.address);

    let lightClientManager = await ethers.getContractAt("LightClientManager",LightClientManagerProxy.address);

    await  lightClientManager.upgradeTo(LightClientManager.address);

    console.log("LightNodeUp ok")
    
}

module.exports.tags = ['LightClientManagerUp']
