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

    console.log("LightNodeUp success");
};

module.exports.tags = ["LightNodeUp"];
