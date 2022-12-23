const {task} = require("hardhat/config");

module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("FeeCenter");

    console.log("Fee center address:", proxy.address);

    let feeCenter = await ethers.getContractAt('FeeCenter',proxy.address);

    await (await feeCenter.connect(deployer).setDistributeRate(
        taskArgs.type,
        taskArgs.address,
        taskArgs.rate
    )).wait();

    console.log(`FeeCenter set distributeRate type: ${taskArgs.type}, receiver: ${taskArgs.address}, rate: ${taskArgs.rate} success`)

}