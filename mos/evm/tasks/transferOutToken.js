
module.exports = async (taskArgs) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

   // let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    //console.log("relay address:", proxy.address);

    let token = await ethers.getContractAt("StandardToken", taskArgs.token);

    let mosProxy = await ethers.getContractAt('IMCS',taskArgs.mos);

    let receiveAddress;
    if (taskArgs.address === ""){
        receiveAddress = deployer.address;
    }else {
        receiveAddress = taskArgs.token
    }

    if (taskArgs.token === "0x0000000000000000000000000000000000000000"){
        await (await mosProxy.connect(deployer).transferOutNative(
            receiveAddress,
            taskArgs.chain,
            {value:taskArgs.value}
        )).wait();

    }else {
        await (await token.connect(deployer).approve(
            taskArgs.mos,
            taskArgs.value
        )).wait();

        await (await mosProxy.connect(deployer).transferOutToken(
            taskArgs.token,
            receiveAddress,
            taskArgs.value,
            taskArgs.chain,
            {
                gasLimit:210000
            }
        )).wait();

    }


    console.log(`MAPCrossChainService transfer out token ${taskArgs.token} ${taskArgs.value} to chain ${taskArgs.chain} ${receiveAddress} successful`);


}