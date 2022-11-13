
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPOmnichainServiceProxy")

    console.log("mos address:", proxy.address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceRelayV2',proxy.address);

    await (await mos.connect(deployer).setDistributeRate(
        taskArgs.type,
        taskArgs.address,
        taskArgs.rate
    )).wait();


    console.log(`mos set distribute ${taskArgs.type} rate ${taskArgs.rate} to ${taskArgs.address} success`)
}