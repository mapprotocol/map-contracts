const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });

module.exports = async function ({ ethers, deployments }) {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    await deploy("Provable", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "Provable",
    });

    let provable = await deployments.get("Provable");

    console.log("Provable success：", provable.address);
};

module.exports.tags = ["Provable"];
