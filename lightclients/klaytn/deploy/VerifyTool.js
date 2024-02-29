const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });

module.exports = async function ({ ethers, deployments }) {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    await deploy("VerifyTool", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "VerifyTool",
    });

    let verifyTool = await deployments.get("VerifyTool");

    console.log("VerifyTool success：", verifyTool.address);
};

module.exports.tags = ["VerifyTool"];
