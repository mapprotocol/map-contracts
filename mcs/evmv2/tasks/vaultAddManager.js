
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    //let proxy = await hre.deployments.get("MAPVaultToken");
    let manager = taskArgs.manager;
    if (taskArgs.manager === "relay") {
        let proxy = await ethers.getContract('MAPOmnichainServiceProxyV2');
        manager = proxy.address;
    }

    let vaultToken = await ethers.getContractAt('VaultTokenV2', taskArgs.vault);

    await (await vaultToken.connect(deployer).addManager(manager)).wait();
    console.log(`MAPVaultToken ${taskArgs.vault} add manager ${manager} success`)

}