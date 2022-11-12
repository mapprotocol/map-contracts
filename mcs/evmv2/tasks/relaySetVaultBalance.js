
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    let relayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    await (await relayProxy.connect(deployer).setVaultBalance(
        taskArgs.chain,
        taskArgs.token,
        taskArgs.balance
    )).wait()
    console.log("MAPCrossChainServiceRelay set relay to mcs setVaultBalance success")

}