
module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address)

    let mcsid = taskArgs.mcsids.split(",");
    let tokendecimals = taskArgs.tokendecimals.split(",");

    if (mcsid.length === tokendecimals.length){
        for (let i = 0; i < mcsid.length; i++){
            await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(
                taskArgs.tokenaddress,
                mcsid[i],
                tokendecimals[i]
            )).wait()
        }
        console.log("MAPCrossChainServiceRelay set mcs token decimals success")
    }else{


        console.error("MAPCrossChainServiceRelay parameter error")

    }


}