module.exports = async ({ getNamedAccounts, deployments }) => {
    const { deployer } = await getNamedAccounts();
    const { deploy } = deployments;

    await deploy("LightNode", {
        from: deployer,
        args: [],
        log: true,
        contract: "LightNode",
    });

    let lightNode = await deployments.get("LightNode");

    let lightNodeProxy = await deployments.get("LightNodeProxy");

    const LightNode = await ethers.getContractFactory("LightNode");

    let proxy = LightNode.attach(lightNodeProxy.address);

    console.log("implementation before: ", await proxy.getImplementation());

    await (await proxy.upgradeTo(lightNode.address)).wait();

    console.log("implementation after: ", await proxy.getImplementation());
};
module.exports.tags = ["Upgrade"];
