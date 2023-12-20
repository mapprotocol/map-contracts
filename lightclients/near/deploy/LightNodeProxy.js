const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require("../test/utils/borsh");

let network = process.env.NETWORK;
module.exports = async ({ getNamedAccounts, deployments, ethers }) => {
    const { deployer } = await getNamedAccounts();
    const { deploy } = deployments;

    let lightNode = await deployments.get("LightNode");

    const iface = new ethers.utils.Interface(["function initialize(address _owner, bytes[2] memory initDatas)"]);

    let block;
    let validators;
    if ("testnet" == network.toString().toLowerCase()) {
        block = "0x" + borshify(require("../scripts/data/testnet/block.json")).toString("hex");
        validators =
            "0x" +
            borshifyInitialValidators(require("../scripts/data/testnet/validators.json").next_bps).toString("hex");
    } else {
        block = "0x" + borshify(require("../scripts/data/mainnet/block.json")).toString("hex");
        validators =
            "0x" +
            borshifyInitialValidators(require("../scripts/data/mainnet/validators.json").next_bps).toString("hex");
    }

    let arr = [validators, block];
    let data = iface.encodeFunctionData("initialize", [deployer, arr]);

    await deploy("LightNodeProxy", {
        from: deployer,
        args: [lightNode.address, data],
        log: true,
        contract: "LightNodeProxy",
        gasLimit: 20000000,
    });
};
module.exports.tags = ["Proxy"];
module.exports.dependencies = ["LightNode"];
