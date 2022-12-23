
module.exports = async (taskArgs) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deposit address:",deployer.address);

    let mos = await ethers.getContractAt('IMOSV2', taskArgs.mos);

    let address = taskArgs.address;
    if (taskArgs.address === "") {
        address = deployer.address;
    }

    if (taskArgs.token === "0x0000000000000000000000000000000000000000") {

        await (await mos.connect(deployer).depositNative(
            address,
            {value:taskArgs.value}
        )).wait();

    }else {
        let token = await ethers.getContractAt("IERC20", taskArgs.token);
        await (await token.connect(deployer).approve(
            taskArgs.mos,
            taskArgs.value
        )).wait();

        await (await mos.connect(deployer).depositToken(
            taskArgs.token,
            address,
            taskArgs.value
        )).wait();

    }

    console.log(`deposit token ${taskArgs.token} ${taskArgs.value} to ${address} successful`);
}