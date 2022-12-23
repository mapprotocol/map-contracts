module.exports = async (taskArgs, hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    await deploy('VaultTokenV2', {
        from: deployer.address,
        args: [taskArgs.token, taskArgs.name, taskArgs.symbol],
        log: true,
        contract: 'VaultTokenV2',
    })

    let vault = await ethers.getContract('VaultTokenV2');

    console.log(`VaultTokenV2 ${taskArgs.symbol} address: ${vault.address}`);
}