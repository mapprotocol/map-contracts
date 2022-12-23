
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPOmnichainServiceProxyV2");

    console.log("mos address:", proxy.address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceV2',proxy.address);

    let tokens = taskArgs.token.split(",");
    if (taskArgs.mintable) {
        await (await mos.connect(deployer).addMintableToken(
            tokens
        )).wait();

        console.log(`mos set token ${taskArgs.token} mintable ${taskArgs.mintable} success`);
    } else {
        await (await mos.connect(deployer).removeMintableToken(
            tokens
        )).wait();

        console.log(`mos set token ${taskArgs.token} mintable ${taskArgs.mintable}  success`);
    }

}