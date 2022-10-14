
let mcssProxyAddress = "0xDFeb47E534FF1DC9EEAdc7388c6460DB8a036753"

let ethUsdt = "0xdBf63a81d44DA9645498E371A856F9754F4f2c2B";

let mapBridageAddress = "0x40D6ff6337D6885B5975DCA5682592856846B277";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let mcssProxy = await ethers.getContractAt('MapCrossChainService',mcssProxyAddress);


    await (await mcssProxy.connect(deployer).setBridge(mapBridageAddress,213)).wait();

    await (await mcssProxy.connect(deployer).setCanBridgeToken(ethUsdt,213,true)).wait();
    await (await mcssProxy.connect(deployer).setCanBridgeToken(ethUsdt,1313161555,true)).wait();

    console.log("MapCrossChainService set success");

}

module.exports.tags = ['MapCrossChainServiceProxySet']