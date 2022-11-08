
module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
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

    console.log("MapCrossChainService address:",mcss.address);


    let data;
    await ( data = await mcss.initialize(taskArgs.wrapped, taskArgs.lightnode)).wait();

    console.log("MapCrossChainService initialize success");

    await deploy('MapCrossChainServiceProxy', {
        from: deployer.address,
        args: [mcss.address,data.data],
        log: true,
        contract: 'MapCrossChainServiceProxy',
    })

    let mcssP = await ethers.getContract('MapCrossChainServiceProxy');

    console.log("MapCrossChainServiceProxy address:",mcssP.address)


}