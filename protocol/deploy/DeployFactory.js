
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('DeployFactory', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'DeployFactory',
        deterministicDeployment: false
    })

    let deployFactory = await ethers.getContract('DeployFactory');

    console.log("deployFactory address:", deployFactory.address)

}

module.exports.tags = ['DeployFactory']