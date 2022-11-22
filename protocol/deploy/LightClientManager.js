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

    let data = LightClientManager.interface.encodeFunctionData("initialize", []);

    await deploy('LightClientManagerProxy', {
        from: deployer.address,
        args: [LightClientManager.address,data],
        log: true,
        contract: 'LightClientManagerProxy',
    })

    let LightClientManagerProxy = await ethers.getContract('LightClientManagerProxy');

    console.log("LightClientManagerProxy",LightClientManagerProxy.address)

}

module.exports.tags = ['LightClientManager']
