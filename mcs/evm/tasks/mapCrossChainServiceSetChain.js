
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MapCrossChainServiceProxy");

    console.log(proxy.address)

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',proxy.address);

    await (await mcssProxy.connect(deployer).setChain(
        taskArgs.name,
        taskArgs.chain
    )).wait();

    console.log(`MapCrossChainService setChain successfully `);

}