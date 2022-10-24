
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);


    await (await mcssRelayProxy.connect(deployer).setBridgeAddress(taskArgs.mcsid,taskArgs.mcsaddr)).wait();

    console.log(`MAPCrossChainServiceRelay set id is ${taskArgs.mcsid} bridgeAddress success`);


}