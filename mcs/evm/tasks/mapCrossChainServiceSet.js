
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MapCrossChainServiceProxy");

    console.log("MapCrossChainService address", proxy.address)

    let mcsProxy = await ethers.getContractAt('MapCrossChainService',proxy.address);


    let relayProxy = await hre.deployments.get("MapCrossChainServiceRelayProxy");

    await (await mcsProxy.connect(deployer).setBridge(taskArgs.relay, taskArgs.chain)).wait();

    console.log(`MapCrossChainService set  relayAddress ${taskArgs.relay} with chain id ${taskArgs.chain} successfully `);

}