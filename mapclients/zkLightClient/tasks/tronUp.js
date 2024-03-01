const TronWeb = require("tronweb");
const { LightNodeProxyMainnetAddress, LightNodeProxyTestAddress } = require("../deployments/TronContract");
require("dotenv").config();
let fs = require("fs");

module.exports = async (taskArgs, hre) => {
    let tronWeb = await getTronWeb(taskArgs.chain);
    let deployer = "0x" + tronWeb.defaultAddress.hex.substring(2);
    console.log("deployer :", tronWeb.address.fromHex(deployer));

    let time = Date.now();
    let dataWrite;
    let lightnodeAddress;
    if (taskArgs.chain === "Tron") {
        let LightNode = await artifacts.readArtifact("LightNode");

        let lightNode = await tronWeb.contract().new({
            abi: LightNode.abi,
            bytecode: LightNode.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });

        console.log(`LightNode main deployed : ${lightNode.address}`);

        lightnodeAddress = "0x" + lightNode.address.substring(2);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(lightnodeAddress),
            evmAddress: lightnodeAddress,
        };

        dataWrite =
            "let LightNodeMainnetAddress" +
            time +
            " = " +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.LightNodeMainnetAddress" +
            time +
            " = LightNodeMainnetAddress" +
            time;
        fs.appendFileSync("./TronContract.js", dataWrite);
        console.log(`deploy LightNode write ${addressList} in mainnet success`);
    } else if (taskArgs.chain === "TronTest") {
        let LightNode = await artifacts.readArtifact("LightNode");

        let lightNode = await tronWeb.contract().new({
            abi: LightNode.abi,
            bytecode: LightNode.bytecode,
            feeLimit: 15000000000,
            callValue: 0,
            parameters: [],
        });

        console.log(`LightNode test deployed: ${lightNode.address}`);

        lightnodeAddress = "0x" + lightNode.address.substring(2);

        let addressList = {
            tronAddress: tronWeb.address.fromHex(lightnodeAddress),
            evmAddress: lightnodeAddress,
        };

        dataWrite =
            "\n" +
            "let LightNodeTestAddress" +
            time +
            " = " +
            JSON.stringify(addressList) +
            "\n" +
            "module.exports.LightNodeTestAddress" +
            time +
            " = LightNodeTestAddress" +
            time +
            "\n";
        fs.appendFileSync("./tasks/TronContract.js", dataWrite);
        console.log(`deploy LightNode write ${addressList.tronAddress} in test success`);
    }

    let LightNode = await artifacts.readArtifact("LightNode");
    let proxyAddress;
    if (taskArgs.chain === "Tron") {
        proxyAddress = LightNodeProxyMainnetAddress.evmAddress;
    } else if (taskArgs.chain === "TronTest") {
        proxyAddress = LightNodeProxyTestAddress.evmAddress;
    }

    let lightNodeProxy = await tronWeb.contract(LightNode.abi, tronWeb.address.fromHex(proxyAddress));

    await lightNodeProxy.upgradeTo(lightnodeAddress).send();

    console.log("LightNodeUp success");
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
