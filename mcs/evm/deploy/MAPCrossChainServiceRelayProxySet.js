
let mcssRelayProxyAddress = "0x40D6ff6337D6885B5975DCA5682592856846B277"

let mapUsdt = "0xa2FD5Ad95c1F2fC374dF775ad0889eab6d587015";

let ethBridageAddress = "0xC435302ee2AE8B2BfAc9520e0A1B083Ce32F6c74";
let nearBridageAddress = "0x6d63732e6d61703030312e746573746e6574";

let feeCenter = "0xCb6F2fB431b5A964763C9ad14FaA2d5802d27dB0";
let tokenRegister = "0x9B12Acf2C97Fc939f29Ee0BD0083Ec29C4F00BE3";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);



    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',mcssRelayProxyAddress);


    await (await mcssRelayProxy.connect(deployer).setFeeCenter(feeCenter)).wait();

    await (await mcssRelayProxy.connect(deployer).setTokenRegister(tokenRegister)).wait();

    await (await mcssRelayProxy.connect(deployer).setIdTable(1313161555,1)).wait();

    await (await mcssRelayProxy.connect(deployer).setChainId(1313161555)).wait();

    await (await mcssRelayProxy.connect(deployer).setBridageAddress(1313161555,nearBridageAddress)).wait();

    await (await mcssRelayProxy.connect(deployer).setBridageAddress(34434,ethBridageAddress)).wait();

    console.log("set success");

    //MAPCrossChainServiceRelay set setTokenOtherChainDecimals
    await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(mapUsdt,34434,18));
    await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(mapUsdt,212,18));
    await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(mapUsdt,1313161555,24));
    console.log("setTokenOtherChainDecimals set success");
    //MAPCrossChainServiceRelay set setVaultBalance
    await (await mcssRelayProxy.connect(deployer).setVaultBalance(34434,mapUsdt,"10000000000000000000000000000")).wait();
    await (await mcssRelayProxy.connect(deployer).setVaultBalance(1313161555,mapUsdt,"10000000000000000000000000000")).wait();

    console.log("setVaultBalance set success");

}

module.exports.tags = ['MAPCrossChainServiceRelayProxySet']