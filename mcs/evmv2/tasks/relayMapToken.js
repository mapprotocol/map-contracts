
function stringToHex(str) {
    return str.split("").map(function(c) {
        return ("0" + c.charCodeAt(0).toString(16)).slice(-2);
    }).join("");
}


module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let tokenmanager = await hre.deployments.get("TokenRegisterProxy");

    console.log("Token manager address:", tokenmanager.address);

    let manager = await ethers.getContractAt('TokenRegisterV2', tokenmanager.address);

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