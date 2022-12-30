module.exports = async (taskArgs, hre) => {
   // const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    let TimeLock = await ethers.getContractFactory('TimelockController');

    let proposers = taskArgs.proposer.split(",");
    let executors = taskArgs.executor.split(",");

    let TimeLockBytescode = await TimeLock.interface.encodeDeploy([
        taskArgs.timenum,
        proposers,
        executors,
        taskArgs.admin
    ])

    let createCode = TimeLock.bytecode + TimeLockBytescode.slice(2)
    console.log(createCode)

    let deployFactory = await ethers.getContractAt('DeployFactory',taskArgs.factory);

    await (await deployFactory.connect(deployer).deployFactory(taskArgs.salt,createCode,taskArgs.valuenum)).wait();

    let timeLockAddress = await deployFactory.connect(deployer).getAddress(taskArgs.salt);

    console.log("TimeLock address:", timeLockAddress);
}