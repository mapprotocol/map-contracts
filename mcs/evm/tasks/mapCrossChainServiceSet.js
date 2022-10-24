
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MapCrossChainServiceProxy");

    console.log(proxy.address)

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',proxy.address);

    await (await mcssProxy.connect(deployer).setBridge(taskArgs.relayaddress,taskArgs.chainid)).wait();

    console.log(`MapCrossChainService set  relayAddress to ${taskArgs.chainid} successfully `);

}