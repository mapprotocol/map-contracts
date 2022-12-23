

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('TokenRegisterV2', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'TokenRegisterV2',
    })

    let tokenRegister = await ethers.getContract('TokenRegisterV2');
    console.log("TokenRegisterV2 address:",tokenRegister.address);


    let proxy = await deployments.get("TokenRegisterProxy");

    let tokenRegisterProxy = await ethers.getContractAt('TokenRegisterV2', proxy.address);

    await (await tokenRegisterProxy.upgradeTo(tokenRegister.address)).wait();

    console.log("TokenRegister up success")
    

}

module.exports.tags = ['TokenRegisterUp']