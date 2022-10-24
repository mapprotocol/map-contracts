
module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('StandardToken', {
        from: deployer.address,
        args: [taskArgs.name,taskArgs.symbol],
        log: true,
        contract: 'StandardToken',
    })

    let mccToken = await ethers.getContract('StandardToken');

    console.log("MakaluCrossChainToken address:",mccToken.address);

}