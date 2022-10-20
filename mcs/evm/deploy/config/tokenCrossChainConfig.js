//MapCrossChainService is deployed on an evm-compatible chain

//bsc test chain id
let mcsChainId = "34434";
//bsc test chain MapCrossChainService contract address
let mcsAddress = "0x10aBb7c593136239A7930FC985B2d415157C4a93";
//bsc test chain cross chain token address
let mcsMotTokenAddress = "0xF2bC92ae38cBd7E766aed3e1C956C3fcA6C583af";
//Accuracy of cross-chain tokens
let mcsMotTokenDecimals = "18"




//MAPCrossChainServiceRelay is deployed on the MAP chain

//Makalu chain id
let relayChainId = "213";
// Makalu deploy FeeCenter contract address
let relayFeeCenterAddress = "0xB03E7279b52AD28830aCdA506FF34D36590292BA";
// Makalu deploy TokenRegister contract address
let relayTokenRegisterAddress ="0xca9B8BD965fc251885916fbfba20C4140c16c76B";
//Makalu MAPCrossChainServiceRelay contract address
let relayAddress = "0xf5AB4F4cD556BeD5ddb4A0493C569716Ae9ace26";
//Makalu test chain cross chain token address
let ralayMotTokenAddress = "0xd629533aF9F6634859E8Ba865092809244E51014";
//Accuracy of cross-chain tokens
let ralayMotTokenDecimals = "18";

//Makalu test chain cross chain vault token address
let ralayVaultTokenAddress = "0xe5629D35af184012949f0CAd44079f13073f3493";
//Makalu test chain cross chain vault token name
let relayVaultTokenName = "MOT Vault Token";
//Makalu test chain cross chain vault token symbol
let relayVaultTokenSymbol = "MOTVT";
//The minimum handling charge
let minFee = "1000000000000000"
//The maximum fee charged
let maxFee = "10000000000000000000"
// Fee ratio /10000
let rateFee = "1500"
//Allowed token cross-chain quota, initial value
let relayToMcsLimit = "1000000000000000000000000";




//near chain id
let nearChainId = "";
//near test executeId bytes
let nearExecuteId = "";
//near test chain cross chain token bytes
let nearMotTokenAddress = "";
//Accuracy of cross-chain tokens
let nearMotTokenDecimals = "24";
//Allowed token cross-chain quota, initial value
let relayToNearLimit = "1000000000000000000000000";


module.exports = {
    mcsChainId,
    mcsAddress,
    mcsMotTokenAddress,
    mcsMotTokenDecimals,
    relayChainId,
    relayFeeCenterAddress,
    relayTokenRegisterAddress,
    relayAddress,
    ralayMotTokenAddress,
    ralayMotTokenDecimals,
    ralayVaultTokenAddress,
    relayVaultTokenName,
    relayVaultTokenSymbol,
    relayToMcsLimit,
    minFee,
    maxFee,
    rateFee,
    nearChainId,
    nearExecuteId,
    nearMotTokenAddress,
    nearMotTokenDecimals,
    relayToNearLimit
}