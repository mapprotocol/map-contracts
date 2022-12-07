
const {task} = require("hardhat/config");

module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('MintableToken', {
        from: deployer.address,
        args: [taskArgs.name, taskArgs.symbol, taskArgs.decimals],
        log: true,
        contract: 'MintableToken',
    })

    let token = await ethers.getContract('MintableToken');

    console.log(`Deply token '${taskArgs.symbol}' address:`, token.address);

    if (taskArgs.balance > 0) {
        balance = ethers.BigNumber.from(taskArgs.balance).mul(ethers.BigNumber.from("1000000000000000000"))

        await token.mint(deployer.address, balance.toString())

        console.log(`Mint '${taskArgs.name}' Token ${taskArgs.balance} ${taskArgs.symbol}`);
    }

}