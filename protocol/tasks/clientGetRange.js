
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];
    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());

    managerAddress = taskArgs.manager;
    if (taskArgs.manager == "") {
        let proxy = await hre.deployments.get("LightClientManagerProxy");
        managerAddress = proxy.address;
    }
    console.log("light client manager address:", managerAddress);
    let manager = await ethers.getContractAt('LightClientManager', managerAddress);

    let lightnode = await manager.lightClientContract(taskArgs.chain);

    let header = await manager.headerHeight(taskArgs.chain);

    let range = await manager.verifiableHeaderRange(taskArgs.chain);

    console.log(`chain ${taskArgs.chain} address(${lightnode}) height(${header}) verifiable header min(${range[0]}), max(${range[1]})`);
}