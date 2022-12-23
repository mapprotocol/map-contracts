
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let tokenmanager = await hre.deployments.get("TokenRegisterProxy");

    console.log("Token manager address:", tokenmanager.address);

    let manager = await ethers.getContractAt('TokenRegisterV2', tokenmanager.address);

    await (await manager.connect(deployer).registerToken(
        taskArgs.token,
        taskArgs.vault,
        taskArgs.mintable
    )).wait()

    console.log(`register token ${taskArgs.token} success`)
}