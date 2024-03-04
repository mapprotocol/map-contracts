const TronWeb = require("tronweb");
require("dotenv").config();

exports.deploy_contract = async function(artifacts, name, args, network) {
    let c = await artifacts.readArtifact(name);
    let tronWeb = await getTronWeb(network)
    let contract_instance = await tronWeb.contract().new({
        abi: c.abi,
        bytecode: c.bytecode,
        feeLimit: 15000000000,
        callValue: 0,
        parameters: args,
    });

    let contract_address = tronWeb.address.fromHex(contract_instance.address);
    console.log(`${name} deployed on: ${contract_address}`);
    return (contract_address,'0x' + contract_instance.address.substring(2));
}

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
