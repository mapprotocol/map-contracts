const {task} = require("hardhat/config");

module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("FeeCenter");

    console.log("fee center address:", proxy.address);

    let feeCenter = await ethers.getContractAt('FeeCenter',proxy.address);

    await (await feeCenter.connect(deployer).setTokenVault(
        taskArgs.token,
        taskArgs.vault
    )).wait();

    console.log(`FeeCenter bind token ${taskArgs.token} and vault ${taskArgs.vault} success`)


}