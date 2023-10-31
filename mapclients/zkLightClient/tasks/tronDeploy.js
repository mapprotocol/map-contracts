const TronWeb = require("tronweb");
const initializeMainnet = require("../deploy/configMainnet");
const initializeTest = require("../deploy/configTest");
const {
    VerifyToolMainnetAddress,
    VerifyToolTestAddress,
    VerifierMainnetAddress,
    VerifierTestAddress,
    LightNodeMainnetAddress,
    LightNodeTestAddress,
    LightNodeProxyMainnetAddress,
    LightNodeProxyTestAddress,
} = require("./TronContract");
require("dotenv").config();
let fs = require("fs");

module.exports = async (taskArgs, hre) => {
    let tronWeb = await getTronWeb(taskArgs.chain);
    let deployer = "0x" + tronWeb.defaultAddress.hex.substring(2);
    console.log("deployer :", tronWeb.address.fromHex(deployer));

    let dataWrite;
    if (taskArgs.chain === "Tron" && VerifyToolMainnetAddress === undefined) {
        let VerifyTool = await artifacts.readArtifact("VerifyTool");
        let verifyTool = await tronWeb.contract().new({
            abi: VerifyTool.abi,
            bytecode: VerifyTool.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });
        console.log(`VerifyTool main deploy: ${verifyTool.address}`);

        let verifyToolAddress = "0x" + verifyTool.address.substring(2);
        console.log("verifyToolMain:", verifyToolAddress);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(verifyToolAddress),
            evmAddress: verifyToolAddress,
        };

        dataWrite =
            "\n" +
            "let VerifyToolMainnetAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.VerifyToolMainnetAddress = VerifyToolMainnetAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy write ${addressList.tronAddress} in mainnet success`);
    } else if (taskArgs.chain === "TronTest" && VerifyToolTestAddress === undefined) {
        let VerifyTool = await artifacts.readArtifact("VerifyTool");
        let verifyTool = await tronWeb.contract().new({
            abi: VerifyTool.abi,
            bytecode: VerifyTool.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });
        console.log(`VerifyTool test deployed on: ${verifyTool.address}`);

        let verifyToolAddress = "0x" + verifyTool.address.substring(2);
        console.log("verifyToolTest:", verifyToolAddress);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(verifyToolAddress),
            evmAddress: verifyToolAddress,
        };

        dataWrite =
            "\n" +
            "let VerifyToolTestAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.VerifyToolTestAddress = VerifyToolTestAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy write ${addressList.tronAddress} in test success`);
    }

    if (taskArgs.chain === "Tron" && VerifierMainnetAddress === undefined) {
        let Verifier = await artifacts.readArtifact("Verifier");
        let verifier = await tronWeb.contract().new({
            abi: Verifier.abi,
            bytecode: Verifier.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });
        console.log(`verifier main deploy: ${verifier.address}`);

        let verifierAddress = "0x" + verifier.address.substring(2);
        console.log("verifyToolMain:", verifierAddress);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(verifierAddress),
            evmAddress: verifierAddress,
        };

        dataWrite =
            "\n" +
            "let VerifierMainnetAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.VerifierMainnetAddress = VerifierMainnetAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy Verifier write ${addressList.tronAddress} in mainnet success`);
    } else if (taskArgs.chain === "TronTest" && VerifierTestAddress === undefined) {
        let Verifier = await artifacts.readArtifact("Verifier");
        let verifier = await tronWeb.contract().new({
            abi: Verifier.abi,
            bytecode: Verifier.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });
        console.log(`verifier main deploy: ${verifier.address}`);

        let verifierAddress = "0x" + verifier.address.substring(2);
        console.log("verifyToolMain:", verifierAddress);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(verifierAddress),
            evmAddress: verifierAddress,
        };

        dataWrite =
            "\n" +
            "let VerifierTestAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.VerifierTestAddress = VerifierTestAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy Verifier write ${addressList.tronAddress} in test success`);
    }

    if (taskArgs.chain === "Tron" && LightNodeMainnetAddress === undefined) {
        let LightNode = await artifacts.readArtifact("LightNode");

        let lightNode = await tronWeb.contract().new({
            abi: LightNode.abi,
            bytecode: LightNode.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });

        console.log(`LightNode main deployed : ${lightNode.address}`);

        let lightnodeAddress = "0x" + lightNode.address.substring(2);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(lightnodeAddress),
            evmAddress: lightnodeAddress,
        };

        dataWrite =
            "\n" +
            "let LightNodeMainnetAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.LightNodeMainnetAddress = LightNodeMainnetAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy LightNode write ${addressList.tronAddress} in mainnet success`);
    } else if (taskArgs.chain === "TronTest" && LightNodeTestAddress === undefined) {
        let LightNode = await artifacts.readArtifact("LightNode");

        let lightNode = await tronWeb.contract().new({
            abi: LightNode.abi,
            bytecode: LightNode.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });

        console.log(`LightNode test deployed: ${lightNode.address}`);

        let lightnodeAddress = "0x" + lightNode.address.substring(2);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(lightnodeAddress),
            evmAddress: lightnodeAddress,
        };

        dataWrite =
            "\n" +
            "let LightNodeTestAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.LightNodeTestAddress = LightNodeTestAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy LightNode write ${addressList.tronAddress} in test success`);
    }

    let initializeData;
    let verifyToolAddress;
    let verifierAddress;
    let lightnodeAddress;
    if (taskArgs.chain === "Tron") {
        initializeData = initializeMainnet;
        verifyToolAddress = VerifyToolMainnetAddress.evmAddress;
        verifierAddress = VerifierTestAddress.evmAddress;
        lightnodeAddress = LightNodeMainnetAddress.evmAddress;
    } else if (taskArgs.chain === "TronTest") {
        initializeData = initializeTest;
        verifyToolAddress = VerifyToolTestAddress.evmAddress;
        verifierAddress = VerifierTestAddress.evmAddress;
        lightnodeAddress = LightNodeTestAddress.evmAddress;
    } else {
        console.log("initializeData network error");
    }

    console.log(lightnodeAddress);

    let validatorsInfo = initializeData.validatorsInfo;

    let validatorsCount = initializeData.validatorsCount;

    let epoch = initializeData.epoch;

    let epochSize = initializeData.epochSize;

    let LightClient = await ethers.getContractFactory("LightNode");

    let data = LightClient.interface.encodeFunctionData("initialize", [
        validatorsInfo,
        validatorsCount,
        epoch,
        epochSize,
        verifyToolAddress,
        verifierAddress,
    ]);
    console.log("initialize success");

    if (taskArgs.chain === "Tron" && LightNodeProxyMainnetAddress === undefined) {
        let LightNodeProxy = await artifacts.readArtifact("LightNodeProxy");
        let lightNodeProxy = await tronWeb.contract().new({
            abi: LightNodeProxy.abi,
            bytecode: LightNodeProxy.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [lightnodeAddress, data],
        });
        console.log(`LightNodeProxy main deployed : ${lightNodeProxy.address}`);

        let lightNodeProxyAddress = "0x" + lightNodeProxy.address.substring(2);

        console.log(lightNodeProxyAddress);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(lightNodeProxyAddress),
            evmAddress: lightNodeProxyAddress,
        };

        dataWrite =
            "\n" +
            "let LightNodeProxyMainnetAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.LightNodeProxyMainnetAddress = LightNodeProxyMainnetAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy LightNodeProxy write ${addressList.tronAddress} in mainnet success`);
    } else if (taskArgs.chain === "TronTest" && LightNodeProxyTestAddress === undefined) {
        let LightNodeProxy = await artifacts.readArtifact("LightNodeProxy");
        let lightNodeProxy = await tronWeb.contract().new({
            abi: LightNodeProxy.abi,
            bytecode: LightNodeProxy.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [lightnodeAddress, data],
        });
        console.log(`LightNodeProxy main deployed : ${lightNodeProxy.address}`);

        let lightNodeProxyAddress = "0x" + lightNodeProxy.address.substring(2);

        console.log(lightNodeProxyAddress);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(lightNodeProxyAddress),
            evmAddress: lightNodeProxyAddress,
        };

        dataWrite =
            "\n" +
            "let LightNodeProxyTestAddress =" +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.LightNodeProxyTestAddress = LightNodeProxyTestAddress" +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy LightNodeProxy write ${addressList.tronAddress} in test success`);
    }
};

async function getTronWeb(network) {
    if (network === "Tron" || network === "TronTest") {
        if (network === "Tron") {
            return new TronWeb(
                "https://api.trongrid.io/",
                "https://api.trongrid.io/",
                "https://api.trongrid.io/",
                process.env.TRON_PRIVATE_KEY
            );
        } else {
            return new TronWeb(
                "https://api.nileex.io/",
                "https://api.nileex.io/",
                "https://api.nileex.io/",
                process.env.TRON_PRIVATE_KEY
            );
        }
    } else {
        throw "unsupport network";
    }
}
