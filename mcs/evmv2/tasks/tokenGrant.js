
module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let token = await ethers.getContractAt('MintableToken', taskArgs.token);

    console.log("Mintable Token address:",token.address);

    let minter = taskArgs.minter;
    if (taskArgs.minter === "mos") {
        let proxy = await ethers.getContract('MAPOmnichainServiceProxyV2');
        minter = proxy.address;
    }

    await token.grantRole("0x9f2df0fed2c77648de5860a4cc508cd0818c85b8b8a1ab4ceeef8d981c8956a6", minter)

    console.log("Grant token ", token.address, " to address", minter)
}