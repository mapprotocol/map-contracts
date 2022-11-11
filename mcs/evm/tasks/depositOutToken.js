
module.exports = async (taskArgs) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let token = await ethers.getContractAt("StandardToken", taskArgs.token);

    let mosProxy = await ethers.getContractAt('MapCrossChainService',taskArgs.mos);

    let receiveAddress;
    if (taskArgs.address === ""){
        receiveAddress = deployer.address;
    }else {
        receiveAddress = taskArgs.token
    }

    if (taskArgs.token === "0x0000000000000000000000000000000000000000"){
        await (await mosProxy.connect(deployer).depositOutNative(
            deployer.address,
            receiveAddress,
            {value:taskArgs.value}
        )).wait();

    }else {
        await (await token.connect(deployer).approve(
            taskArgs.mos,
            taskArgs.value
        )).wait();

        await (await mosProxy.connect(deployer).depositOutToken(
            taskArgs.token,
            deployer.address,
            receiveAddress,
            taskArgs.value,
            {
                gasLimit:210000
            }
        )).wait();

    }


    console.log(`MAPCrossChainService transfer out token ${taskArgs.token} ${taskArgs.value} to chain ${taskArgs.chain} ${receiveAddress} successful`);


}