
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPOmnichainServiceProxyV2");

    console.log("mos address", proxy.address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceV2', proxy.address);

    await (await mos.connect(deployer).setLightClient(taskArgs.client)).wait();

    console.log(`mos set  light client ${taskArgs.client} successfully `);

}