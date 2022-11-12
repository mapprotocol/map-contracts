
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let register = await hre.deployments.get("TokenRegister")

    let chains = taskArgs.chains.split(",");
    let decimals = taskArgs.decimals.split(",");

    if (mcsid.length === decimals.length){
        for (let i = 0; i < chains.length; i++){
            await (await register.connect(deployer).setTokenOtherChainDecimals(
                taskArgs.token,
                chains[i],
                decimals[i]
            )).wait()
        }
        console.log("MAPCrossChainServiceRelay set mcs token decimals success")
    }else{
        console.error("MAPCrossChainServiceRelay parameter error")
    }
}