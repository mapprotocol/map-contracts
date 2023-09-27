
const TronWeb = require('tronweb')
const initializeData = require("../deploy/config");
require('dotenv').config();

module.exports = async (taskArgs,hre) => {
    let tronWeb = await getTronWeb(taskArgs.chain);
    let deployer = '0x' + tronWeb.defaultAddress.hex.substring(2);
    console.log("deployer :",tronWeb.address.fromHex(deployer));

    let VerifyTool = await artifacts.readArtifact("VerifyTool")
    let verifyTool = await tronWeb.contract().new({
            abi:VerifyTool.abi,
            bytecode:VerifyTool.bytecode,
            feeLimit:15000000000,
            callValue:0,
            parameters:[]
    });
    console.log(`VerifyTool deployed on: ${verifyTool.address}`);

    let verifyToolAddress = '0x' + verifyTool.address.substring(2);
    console.log(verifyToolAddress)

    let LightNode = await artifacts.readArtifact("LightNode");

    let lightNode = await tronWeb.contract().new({
        abi:LightNode.abi,
        bytecode:LightNode.bytecode,
        feeLimit:15000000000,
        callValue:0,
        parameters:[]
    });

    console.log(`LightNode deployed on: ${lightNode.address}`);

    let lightnodeAddress = '0x' + lightNode.address.substring(2);

    let validatorNum = initializeData.validators;
    let g1List = [];
    let addresss = [];
    let weights = []
    for (let i = 0; i < validatorNum.length; i++){
        let temp = [validatorNum[i].g1_pub_key.x,validatorNum[i].g1_pub_key.y];
        g1List.push(temp);
        addresss.push(validatorNum[i].address);
        weights.push((validatorNum[i].weight));
    }

    let threshold = initializeData.threshold;

    let epoch = initializeData.epoch;

    let epochSize = initializeData.epoch_size;

    let LightClient = await ethers.getContractFactory("LightNode");

    let data = LightClient.interface.encodeFunctionData("initialize", [threshold, addresss, g1List, weights, epoch, epochSize,verifyToolAddress]);
    console.log("initialize success")
    console.log(data)

    let LightNodeProxy = await artifacts.readArtifact("LightNodeProxy");
    let lightNodeProxy = await tronWeb.contract().new({
        abi:LightNodeProxy.abi,
        bytecode:LightNodeProxy.bytecode,
        feeLimit:15000000000,
        callValue:0,
        parameters:[lightnodeAddress,data]
    });
    console.log(`LightNodeProxy deployed on: ${lightNodeProxy.address}`);

    let lightNodeProxyAddress = '0x' + lightNodeProxy.address.substring(2);

    console.log(lightNodeProxyAddress)

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
