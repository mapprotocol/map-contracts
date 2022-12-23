
function stringToHex(str) {
    return str.split("").map(function(c) {
        return ("0" + c.charCodeAt(0).toString(16)).slice(-2);
    }).join("");
}

module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("TokenRegister");

    console.log("token register address:", proxy.address);

    let chaintoken = taskArgs.chaintoken;
    if (taskArgs.chaintoken.substr(0,2) != "0x") {
        chaintoken = "0x" + stringToHex(taskArgs.chaintoken);
    }

    let tokenRegister = await ethers.getContractAt('TokenRegister',proxy.address);

    await (await tokenRegister.connect(deployer).registerToken(
        taskArgs.chain,
        chaintoken,
        taskArgs.token
    )).wait()

    console.log(`TokenRegister register ${taskArgs.token} with chain ${taskArgs.chain} token ${chaintoken} success `)


}