
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    //let proxy = await hre.deployments.get("MAPVaultToken");
    let manager = taskArgs.manager;
    if (taskArgs.manager === "relay") {
        let mcs = await ethers.getContract('MAPCrossChainServiceRelayProxy');
        manager = mcs.address;
    }

    let vaultToken = await ethers.getContractAt('MAPVaultToken', taskArgs.vault);

    await (await vaultToken.connect(deployer).addManager(manager)).wait();
    console.log(`MAPVaultToken ${taskArgs.vault} add manager ${manager} success`)

}