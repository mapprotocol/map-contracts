
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPOmnichainServiceProxyV2");

    console.log("mos address:", proxy.address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceV2',proxy.address);

    let ids = taskArgs.chains.split(",");

    for (let i = 0; i < ids.length; i++){
        await (await mos.connect(deployer).registerChain(
            ids[i],
            taskArgs.type
        )).wait();

        console.log(`mos register type ${taskArgs.type} to chain ${ids[i]} success`);
    }

    console.log("mos registerChain success");
}
