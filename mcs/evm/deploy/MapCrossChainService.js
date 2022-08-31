const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

let lightNodeAddress = "0x80Be41aEBFdaDBD58a65aa549cB266dAFb6b8304";
let mcsRelayAddress = "0x23b51D50782c42Ac2dcda362E5243795205a02a4";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer,wcoin,mapcoin} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('MapCrossChainService', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MapCrossChainService',
    })

    let mcss = await ethers.getContract('MapCrossChainService');

    console.log("MapCrossChainService address:",mcss.address);


    await (await mcss.initialize(wcoin.address,mapcoin.address,lightNodeAddress)).wait();
    console.log("MapCrossChainService initialize success");

    await (await mcss.setBridge(mcsRelayAddress,212)).wait();

    await (await mcss.setCanBridgeToken("0x0000000000000000000000000000000000000000",212,true)).wait();

    await (await mcss.setCanBridgeToken("0x0000000000000000000000000000000000000000",1313161555,true)).wait();


}

module.exports.tags = ['MapCrossChainService']