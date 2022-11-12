module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

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