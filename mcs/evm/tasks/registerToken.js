
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("TokenRegister");

    console.log("token register address:", proxy.address);

    let tokenRegister = await ethers.getContractAt('TokenRegister',proxy.address);

    await (await tokenRegister.connect(deployer).registerToken(
        taskArgs.chain,
        taskArgs.chaintoken,
        taskArgs.token
    )).wait()

    console.log(`TokenRegister register ${taskArgs.token} with chain ${taskArgs.chain} token ${taskArgs.chaintoken} success `)


}