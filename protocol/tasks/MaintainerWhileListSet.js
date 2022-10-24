
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());
    let MaintainerManager = await ethers.getContract('MaintainerManager');

    console.log("MaintainerManager",MaintainerManager.address)

    let MaintainerManagerProxy = await ethers.getContract('MaintainerManagerProxy');
    console.log("MaintainerManagerProxy",MaintainerManagerProxy.address)

    MaintainerManager = await ethers.getContractAt("MaintainerManager",MaintainerManagerProxy.address)

    if (taskArgs.add) {
        await MaintainerManager.addWhiteList(taskArgs.address);
    }else{
        await MaintainerManager.removeWhiteList(taskArgs.address);
    }

    console.log("success")
}