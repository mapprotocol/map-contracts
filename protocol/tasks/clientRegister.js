
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];
    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let proxy = await ethers.getContract('LightClientManagerProxy');

    console.log("Light Clinet Manager Proxy", proxy.address)

    let manager = await ethers.getContractAt("LightClientManager", proxy.address)

    await manager.register(taskArgs.chain, taskArgs.contract);

    console.log(`Register ${taskArgs.chain} light client ${taskArgs.contract} successfully`)
}