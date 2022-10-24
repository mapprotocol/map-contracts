
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('MapCrossChainService', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MapCrossChainService',
    })

    let mcss = await ethers.getContract('MapCrossChainService');


    console.log("MapCrossChainService up address:",mcss.address);

    let proxy = await deployments.get("MapCrossChainServiceProxy");

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',proxy.address);

    await (await mcssProxy.upgradeTo(mcss.address)).wait();

    console.log("MapCrossChainService up success")

}

module.exports.tags = ['MapCrossChainServiceProxyUp']