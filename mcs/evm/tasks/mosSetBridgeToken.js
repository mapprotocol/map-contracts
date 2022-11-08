
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MapCrossChainServiceProxy");

    console.log("mos address:", proxy.address);

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',proxy.address);

    let id = taskArgs.chains.split(",");

    for (let i = 0; i < id.length; i++){
        await (await mcssProxy.connect(deployer).setCanBridgeToken(
            taskArgs.token,
            id[i],
            true
        )).wait();

        console.log(`MapCrossChainService set token ${taskArgs.token} to chain ${id[i]} success`);
    }

    console.log("MapCrossChainService setCanBridgeToken success");
}
