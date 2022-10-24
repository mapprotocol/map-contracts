
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("FeeCenter");

    let feeCenter = await ethers.getContractAt('FeeCenter',proxy.address);

    await (await feeCenter.connect(deployer).setChainTokenGasFee(
            taskArgs.mcschainid,
            taskArgs.crosstoken,
            taskArgs.minfee,
            taskArgs.maxfee,
            taskArgs.ratefee)
    ).wait();

    console.log("FeeCenter set chainTokenGasFee success")


}