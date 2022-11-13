
function stringToHex(str) {
    return str.split("").map(function(c) {
        return ("0" + c.charCodeAt(0).toString(16)).slice(-2);
    }).join("");
}


module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let proxy = await hre.deployments.get("MAPOmnichainServiceProxyV2")

    console.log("mos address:", proxy.address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceRelay',proxy.address);

    let address = taskArgs.address;
    if (taskArgs.address.substr(0,2) != "0x") {
        address = "0x" + stringToHex(taskArgs.address);
    }

    await (await mos.connect(deployer).registerChain(taskArgs.chain, address, taskArgs.type)).wait();

    console.log(`mos register chain ${taskArgs.chain}  address ${address} success`);


}