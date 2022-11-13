
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let manager = await hre.deployments.get("TokenRegisterV2");

    console.log("Token manager address:", manager.address);

    let chaintoken = taskArgs.chaintoken;
    if (taskArgs.chaintoken.substr(0,2) != "0x") {
        chaintoken = "0x" + stringToHex(taskArgs.chaintoken);
    }

    await (await manager.connect(deployer).mapToken(
        taskArgs.token,
        taskArgs.chain,
        chaintoken,
        taskArgs.decimals
    )).wait()

    console.log(`Token register manager maps chain ${taskArgs.chain} token ${chaintoken} to relay chain token ${taskArgs.mapToken}  success `)
}