
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("TokenRegister");

    let tokenRegister = await ethers.getContractAt('TokenRegister',proxy.address);

    await (await tokenRegister.connect(deployer).regToken(
        taskArgs.crossid,
        taskArgs.crosstoken,
        taskArgs.maptoken
    )).wait()

    console.log("TokenRegister success ")


}