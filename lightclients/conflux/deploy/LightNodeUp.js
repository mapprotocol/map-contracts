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

    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let LightClient= await ethers.getContract('LightNode');

    console.log("LightNode",LightClient.address);

    let LightClientProxy = await ethers.getContract('LightNodeProxy');

    console.log("LightNodeProxy",LightClientProxy.address);

    let lightClientProxy = await ethers.getContractAt("LightNode",LightClientProxy.address);

    await  lightClientProxy.upgradeTo(LightClient.address);

    console.log("LightNodeUp success")

}

module.exports.tags = ['LightNodeUp']
