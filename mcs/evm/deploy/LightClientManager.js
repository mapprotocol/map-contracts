
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);


    await deploy('LightClientManager', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightClientManager',
    })


    let lightNodeManager = await ethers.getContract('LightClientManager');

    console.log("LightClientManager address:",lightNodeManager.address);

}

module.exports.tags = ['LightClientManager']