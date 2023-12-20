module.exports = async ({ getNamedAccounts, deployments }) => {
    const { deployer } = await getNamedAccounts();
    const { deploy } = deployments;
    console.log(deployer);

    await deploy("LightNode", {
        from: deployer,
        args: [],
        log: true,
        contract: "LightNode",
    });
};
module.exports.tags = ["LightNode"];
