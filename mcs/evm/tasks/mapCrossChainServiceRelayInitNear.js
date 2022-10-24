
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    await (await mcssRelayProxy.connect(deployer).setIdTable(taskArgs.nearid,1)).wait();

    await (await mcssRelayProxy.connect(deployer).setChainId(taskArgs.nearid)).wait();

    console.log(`MAPCrossChainServiceRelay init near chain success`);


}