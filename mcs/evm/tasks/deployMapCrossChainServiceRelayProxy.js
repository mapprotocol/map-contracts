
module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
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

    console.log("MAPCrossChainServiceRelay address:",mcssRelay.address);

    let data = await mcssRelay.initialize(taskArgs.weth,taskArgs.maptoken,taskArgs.lightnode);
    console.log("init success");

    await deploy('MAPCrossChainServiceRelayProxy', {
        from: deployer.address,
        args: [mcssRelay.address,data.data],
        log: true,
        contract: 'MAPCrossChainServiceRelayProxy',
    })

    let mcssRelayP = await ethers.getContract('MAPCrossChainServiceRelayProxy');

    console.log("MAPCrossChainServiceRelayProxy address:",mcssRelayP.address);

}