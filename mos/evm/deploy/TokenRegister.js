

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);


    await deploy('TokenRegister', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'TokenRegister',
    })


    let tokenRegister = await ethers.getContract('TokenRegister');

    console.log("tokenRegister address:",tokenRegister.address);


}

module.exports.tags = ['TokenRegister']