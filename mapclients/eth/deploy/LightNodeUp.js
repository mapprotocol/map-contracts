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


    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let lightNode = await ethers.getContract('LightNode');

    let proxy = await  ethers.getContract("LightNodeProxy");

    console.log(lightNode.address)
    console.log(proxy.address)

    let lightNodeProxy = await ethers.getContractAt("LightNode",proxy.address);

    await  lightNodeProxy.upgradeTo(lightNode.address);

    console.log("LightNodeUp ok")
}

module.exports.tags = ['LightNodeUp']
