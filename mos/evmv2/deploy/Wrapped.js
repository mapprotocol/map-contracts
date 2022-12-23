
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);


    await deploy('Wrapped', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'Wrapped',
    })

    let weth = await ethers.getContract('Wrapped');

    console.log("Wrapped address:",weth.address);

}

module.exports.tags = ['Wrapped']