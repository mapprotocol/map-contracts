
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("FeeCenter");

    let feeCenter = await ethers.getContractAt('FeeCenter',proxy.address);

    await (await feeCenter.connect(deployer).setDistributeRate(
        "1",
        taskArgs.vaulttoken,
        taskArgs.ratenumber
    )).wait();


    console.log("FeeCenter set distributeRate success")


}