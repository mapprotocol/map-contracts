
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let manager = await hre.deployments.get("TokenRegisterV2");

    console.log("Token manager address:", manager.address);

    await (await manager.connect(deployer).setTokenFee(
            taskArgs.token,
            taskArgs.chain,
            taskArgs.min,
            taskArgs.max,
            taskArgs.rate)
    ).wait();

    console.log(`Token register manager set token ${taskArgs.token} to chain ${taskArgs.chain} fee success`)


}