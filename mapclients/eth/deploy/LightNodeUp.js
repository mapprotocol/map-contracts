const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
const  initializeData = require('./config');
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

    console.log(lightNode.address)
    let lightNodeProxy = await ethers.getContractAt("LightNode",initializeData.lightNodeProxyAddress);

    await (await  lightNodeProxy.upgradeTo(lightNode.address)).wait();

    console.log("LightNodeUp ok")
}

module.exports.tags = ['LightNodeUp']
