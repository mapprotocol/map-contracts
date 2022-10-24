
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];
    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let LightClientManager = await ethers.getContract('LightClientManager');

    await LightClientManager.register(taskArgs.chain,taskArgs.contract);

    console.log("success")
}