module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);


    await deploy('MAPVaultToken', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPVaultToken',
    })

    let mapVaultToken = await ethers.getContract('MAPVaultToken');

    console.log("MAPVaultToken address:",mapVaultToken.address);

}

module.exports.tags = ['MAPVaultToken']