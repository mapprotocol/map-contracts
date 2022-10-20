//MapCrossChainService is deployed on an evm-compatible chain

//bsc test chain id
let mcsChainId = "34434";
//bsc test MapCrossChainService contract address
let mcsAddress = "0x10aBb7c593136239A7930FC985B2d415157C4a93"


//MAPCrossChainServiceRelay is deployed on the MAP chain

//Makalu chain id
let relayChainId = "213";
// Makalu deploy FeeCenter contract address
let relayFeeCenterAddress = "0xB03E7279b52AD28830aCdA506FF34D36590292BA";
// Makalu deploy TokenRegister contract address
let relayTokenRegisterAddress ="0xca9B8BD965fc251885916fbfba20C4140c16c76B";
//Makalu MAPCrossChainServiceRelay contract address
let relayAddress = "0xf5AB4F4cD556BeD5ddb4A0493C569716Ae9ace26";


//near chain id
let mcsNearChainId = "";
//near test executeId bytes
let nearExecuteId = "";

module.exports = {
    mcsChainId,
    mcsAddress,
    relayChainId,
    relayFeeCenterAddress,
    relayTokenRegisterAddress,
    relayAddress,
    mcsNearChainId,
    nearExecuteId
}