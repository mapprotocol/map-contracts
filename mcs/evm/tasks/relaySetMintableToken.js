
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MapCrossChainServiceRelayProxy");

    console.log("mos relay address:", proxy.address);

    let mosProxy = await ethers.getContractAt('MapCrossChainServiceRelay',proxy.address);

    let tokens = taskArgs.token.split(",");
    if (taskArgs.mintable) {
        await (await mosProxy.connect(deployer).addAuthToken(
            tokens
        )).wait();

        console.log(`MapCrossChainService set token ${taskArgs.token} mintable ${taskArgs.mintable}  success`);

    } else {
        await (await mosProxy.connect(deployer).removeAuthToken(
            tokens
        )).wait();

        console.log(`MapCrossChainService remove token ${taskArgs.token} mintable ${taskArgs.mintable}  success`);
    }
}