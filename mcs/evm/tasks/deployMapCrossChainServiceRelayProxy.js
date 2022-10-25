
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

    let mcsRelay = await ethers.getContract('MAPCrossChainServiceRelay');

    console.log("MAPCrossChainServiceRelay address:",mcsRelay.address);

    let data;
    await ( data = await mcsRelay.initialize(taskArgs.wrapped,taskArgs.wrapped,taskArgs.lightnode)).wait();
    //let data = await mcsRelay.initialize(taskArgs.wrapped, taskArgs.wrapped, taskArgs.lightnode);
    console.log("MAPCrossChainServiceRelay init success");

    await deploy('MAPCrossChainServiceRelayProxy', {
        from: deployer.address,
        args: [mcsRelay.address,data.data],
        log: true,
        contract: 'MAPCrossChainServiceRelayProxy',
    })

    let mcsRelayProxy = await ethers.getContract('MAPCrossChainServiceRelayProxy');

    console.log("MAPCrossChainServiceRelayProxy address:",mcsRelayProxy.address);

}