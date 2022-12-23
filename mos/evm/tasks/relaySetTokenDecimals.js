
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy");

    console.log("MOS address:", proxy.address);

    let mcsProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    console.log("token address:", taskArgs.token);

    let chains = taskArgs.chain.split(",");
    let decimals = taskArgs.decimals.split(",");

    if (chains.length === decimals.length){
        for (let i = 0; i < chains.length; i++){
            await (await mcsProxy.connect(deployer).setTokenOtherChainDecimals(
                taskArgs.token,
                chains[i],
                decimals[i]
            )).wait()

            console.log(`MAPCrossChainServiceRelay set chain ${chains[i]} decimals ${decimals[i]} success`)
        }

    }else{
        console.error("MAPCrossChainServiceRelay parameter error")
    }
}