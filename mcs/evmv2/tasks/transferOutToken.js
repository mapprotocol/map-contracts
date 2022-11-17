
function stringToHex(str) {
    return str.split("").map(function(c) {
        return ("0" + c.charCodeAt(0).toString(16)).slice(-2);
    }).join("");
}

module.exports = async (taskArgs) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let token = await ethers.getContractAt("IERC20", taskArgs.token);

    let mos = await ethers.getContractAt('IMOSV2',taskArgs.mos);

    let address = taskArgs.address;
    if (taskArgs.address === "") {
        address = deployer.address;
    } else {
        if (taskArgs.address.substr(0,2) != "0x") {
            address = "0x" + stringToHex(taskArgs.address);
        }
    }

    if (taskArgs.token === "0x0000000000000000000000000000000000000000"){
        await (await mos.connect(deployer).transferOutNative(
            address,
            taskArgs.chain,
            {value:taskArgs.value}
        )).wait();
    }else {
        await (await token.connect(deployer).approve(
            taskArgs.mos,
            taskArgs.value
        )).wait();

        await (await mos.connect(deployer).transferOutToken(
            taskArgs.token,
            address,
            taskArgs.value,
            taskArgs.chain
        )).wait();

    }

    console.log(`transfer out token ${taskArgs.token} ${taskArgs.value} to chain ${taskArgs.chain} ${address} successful`);
}