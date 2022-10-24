
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);


    await deploy('WETH9', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'WETH9',
    })

    let weth = await ethers.getContract('WETH9');

    console.log("WETH address:",weth.address);

}

module.exports.tags = ['WETH']