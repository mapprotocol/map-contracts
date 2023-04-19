
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];
    console.log(
        "deployer:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());

    console.log("DeployFactory address:", taskArgs.factory);
    let factory = await ethers.getContractAt('IDeployFactory', taskArgs.factory);

    console.log("salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let deployedAddress = await factory.connect(deployer).getAddress(hash);

    console.log(`Get address ${deployedAddress} deployed by factory (${factory.address}) with salt (${taskArgs.salt})`);
}