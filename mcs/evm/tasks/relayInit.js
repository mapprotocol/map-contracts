
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    console.log("set fee center:", taskArgs.feecenter);
    await (await mcssRelayProxy.connect(deployer).setFeeCenter(taskArgs.feecenter)).wait();

    console.log("set token register:", taskArgs.tokenregister);
    await (await mcssRelayProxy.connect(deployer).setTokenRegister(taskArgs.tokenregister)).wait();


}