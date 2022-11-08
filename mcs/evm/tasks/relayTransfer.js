
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    console.log("relay address:", proxy.address);

    let token = await ethers.getContractAt("StandardToken", taskArgs.token);

    let mosProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    await (await token.connect(deployer).approve(
        proxy.address,
        taskArgs.value
    )).wait();

    await (await mosProxy.connect(deployer).transferOutToken(
        taskArgs.token,
        taskArgs.address,
        taskArgs.value,
        taskArgs.chain
    )).wait();

    console.log(`MAPCrossChainService transfer out token ${taskArgs.token} ${taskArgs.value} to chain ${taskArgs.chain} ${taskArgs.address} successful`);


}