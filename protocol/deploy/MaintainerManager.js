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


    await deploy('MaintainerManager', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MaintainerManager',
    })

    let MaintainerManager = await ethers.getContract('MaintainerManager');

    console.log("MaintainerManager",MaintainerManager.address)

    await deploy('MaintainerManagerProxy', {
        from: deployer.address,
        args: [MaintainerManager.address,"0x"],
        log: true,
        contract: 'MaintainerManagerProxy',
    })

    let MaintainerManagerProxy = await ethers.getContract('MaintainerManagerProxy');
    console.log("MaintainerManagerProxy",MaintainerManagerProxy.address)

    MaintainerManagerProxy = await ethers.getContractAt("MaintainerManager",MaintainerManagerProxy.address)
    await MaintainerManagerProxy.initialize();

    console.log("MaintainerManager admin",await MaintainerManagerProxy.getAdmin())
}

module.exports.tags = ['MaintainerManager']
