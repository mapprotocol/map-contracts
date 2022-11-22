

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

    let data = tokenRegister.interface.encodeFunctionData("initialize", []);

    await deploy('TokenRegisterProxy', {
        from: deployer.address,
        args: [tokenRegister.address,data],
        log: true,
        contract: 'TokenRegisterProxy',
    })

    let tokenRegisterProxy = await ethers.getContract('TokenRegisterProxy');
    
    console.log("TokenRegisterProxy address:",tokenRegisterProxy.address);
    

}

module.exports.tags = ['TokenRegisterV2']