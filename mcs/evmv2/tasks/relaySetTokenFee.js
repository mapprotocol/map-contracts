
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let tokenmanager = await hre.deployments.get("TokenRegisterProxy");

    console.log("Token manager address:", tokenmanager.address);

    let manager = await ethers.getContractAt('TokenRegisterV2', tokenmanager.address);

    await (await manager.connect(deployer).setTokenFee(
            taskArgs.token,
            taskArgs.chain,
            taskArgs.min,
            taskArgs.max,
            taskArgs.rate)
    ).wait();

    console.log(`Token register manager set token ${taskArgs.token} to chain ${taskArgs.chain} fee success`)


}