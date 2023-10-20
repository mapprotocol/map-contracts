
const TronWeb = require('tronweb')
const initializeData = require("../deploy/config");
require('dotenv').config();

module.exports = async (taskArgs,hre) => {
    console.log("Tron deploy, network :", hre.network.name);

    let tronWeb = await getTronWeb(hre.network.name);
    console.log("deployer :", tronWeb.defaultAddress);

    let VerifyTool = await artifacts.readArtifact("VerifyTool")
    let verifyTool = await tronWeb.contract().new({
            abi:VerifyTool.abi,
            bytecode:VerifyTool.bytecode,
            feeLimit:1500000000,
            callValue:0,
            parameters:[]
    });

    let verifyToolAddress = '0x' + verifyTool.address.substring(2);

    console.log(`VerifyTool deployed on: ${tronWeb.address.fromHex(verifyTool.address)} (${verifyTool.address.toHex()})`);

    let LightNode = await artifacts.readArtifact("LightNode");

    let lightNode = await tronWeb.contract().new({
        abi:LightNode.abi,
        bytecode:LightNode.bytecode,
        feeLimit:2000000000,
        callValue:0,
        parameters:[]
    });

    let lightnodeAddress = '0x' + lightNode.address.substring(2);

    console.log(`LightNode deployed on: ${tronWeb.address.fromHex(lightNode.address)} (${lightNode.address.toHex()})`);

    let validatorNum = initializeData.validators;
    let g1List = [];
    let addresss = [];
    let weights = []
    for (let i = 0; i < validatorNum.length; i++) {
        let temp = [validatorNum[i].g1_pub_key.x, validatorNum[i].g1_pub_key.y];
        g1List.push(temp);
        addresss.push(validatorNum[i].address);
        weights.push((validatorNum[i].weight));
    }

    let threshold = initializeData.threshold;

    let epoch = initializeData.epoch;

    let epochSize = initializeData.epoch_size;

    let LightClient = await artifacts.readArtifact("LightNode");
    let lightClientInterface = await new tronWeb.utils.ethersUtils.Interface(LightClient.abi)

    let data = lightClientInterface.encodeFunctionData("initialize", [threshold, addresss, g1List, weights, epoch, epochSize, verifyToolAddress]);
    console.log("initialize success");
    // console.log(data);

    let LightNodeProxy = await artifacts.readArtifact("LightNodeProxy");
    let lightNodeProxy = await tronWeb.contract().new({
        abi:LightNodeProxy.abi,
        bytecode:LightNodeProxy.bytecode,
        feeLimit: 6000000000,
        callValue:0,
        parameters:[lightnodeAddress, data]
    });

    console.log(`LightNodeProxy deployed on: ${tronWeb.address.fromHex(lightNodeProxy.address)} (${lightNodeProxy.address.toHex()})`);
}

async function getTronWeb (network) {
    if(network === "Tron" || network === "TronTest"){

        if(network === "Tron") {
            return new TronWeb(
                "https://api.trongrid.io/",
                "https://api.trongrid.io/",
                "https://api.trongrid.io/",
                process.env.TRON_PRIVATE_KEY
            )
        } else {
            return new TronWeb(
                "https://api.nileex.io/",
                "https://api.nileex.io/",
                "https://api.nileex.io/",
                process.env.TRON_PRIVATE_KEY
            )
        }

    } else {
        throw("unsupport network");
    }

}
