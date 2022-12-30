
module.exports = async (taskArgs, hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];
    console.log(deployer.address)
    let path = "../log/" + hre.network.config.chainId + "/" + "gnosisSafe" + taskArgs.executeid;
    const logData = require(path);

    console.log(logData)

    let timeLock = await ethers.getContractAt('TimelockController',taskArgs.timelockaddress);

    await (await timeLock.connect(deployer).execute(
            logData.target,
            logData.value,
            logData.data,
            logData.predecessor,
            logData.salt,
            {
                gasLimit:"2000000"
            }
    )).wait();

    console.log(`${taskArgs.executeid} Execution is complete `)
}