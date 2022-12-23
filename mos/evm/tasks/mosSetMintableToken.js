
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MapCrossChainServiceProxy");

    console.log("MOS address:", proxy.address);

    let mcsProxy = await ethers.getContractAt('MapCrossChainService',proxy.address);

    let tokens = taskArgs.token.split(",");
    if (taskArgs.mintable) {
        await (await mcsProxy.connect(deployer).addAuthToken(
            tokens
        )).wait();

        console.log(`MapCrossChainService set token ${taskArgs.token} mintable ${taskArgs.mintable}  success`);
    } else {
        await (await mcsProxy.connect(deployer).removeAuthToken(
            tokens
        )).wait();

        console.log(`MapCrossChainService remove token ${taskArgs.token} mintable ${taskArgs.mintable}  success`);
    }


}