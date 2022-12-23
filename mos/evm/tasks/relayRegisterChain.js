
function stringToHex(str) {
    return str.split("").map(function(c) {
        return ("0" + c.charCodeAt(0).toString(16)).slice(-2);
    }).join("");
}


module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPCrossChainServiceRelayProxy")

    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',proxy.address);

    let address = taskArgs.address;
    if (taskArgs.address.substr(0,2) != "0x") {
        address = "0x" + stringToHex(taskArgs.address);
    }

    await (await mcssRelayProxy.connect(deployer).setBridgeAddress(taskArgs.chain, address)).wait();

    console.log(`MAPCrossChainServiceRelay register chain ${taskArgs.chain} mos address ${address} success`);


}