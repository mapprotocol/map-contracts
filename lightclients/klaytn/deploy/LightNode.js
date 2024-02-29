const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });

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
        // deterministicDeployment: true,
        // proxy:{
        //     proxyContract:"OpenZeppelinTransparentProxy"
        // }
    });

    let lightNode = await deployments.get("LightNode");

    console.log("LightNode success：", lightNode.address);
};

module.exports.tags = ["LightNode"];
