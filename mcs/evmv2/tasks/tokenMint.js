
module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let token = await ethers.getContractAt('MintableToken', taskArgs.token);

    console.log("Mintable Token address:",token.address);

    await token.mint(deployer.address, taskArgs.amount)

    console.log(`Mint '${taskArgs.token}' Token ${taskArgs.amount} `);
}