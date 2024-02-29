const initializeData = require("./config");

module.exports = async function ({ ethers, deployments }) {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    await deploy("LightNode", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "LightNode",
    });

    let lightNode = await deployments.get("LightNode");
    console.log(lightNode.address);

    let proxy = await deployments.get("LightNodeProxy");
    console.log("light node proxy: ", proxy.address);

    let lightNodeProxy = await ethers.getContractAt("LightNode", proxy.address);

    await (await lightNodeProxy.upgradeTo(lightNode.address)).wait();

    let verifyAddr = await lightNodeProxy.verifyTool();
    console.log("pre verify tool addr", verifyAddr);

    await deploy("VerifyTool", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "VerifyTool",
    });
    let newVerifyTool = await deployments.get("VerifyTool");
    console.log("new verify tool addr", newVerifyTool.address);
    if (verifyAddr !== newVerifyTool.addr) {
        await (await lightNodeProxy.setVerifyTool(newVerifyTool.address)).wait();
    }

    console.log("LightNodeUp success");
};

module.exports.tags = ["LightNodeUp"];
