

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('MAPCrossChainServiceRelay', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPCrossChainServiceRelay',
    })

    let mcssRelay = await ethers.getContract('MAPCrossChainServiceRelay');

    console.log("MAPCrossChainServiceRelay up address:",mcssRelay.address);

    let proxy = await deployments.get("MAPCrossChainServiceRelayProxy")

    let mcssRelayP = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    await (await mcssRelayP.upgradeTo(mcssRelay.address)).wait();

    console.log("MAPCrossChainServiceRelay up success");

}

module.exports.tags = ['MAPCrossChainServiceRelayProxyUp']