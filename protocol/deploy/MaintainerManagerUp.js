const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });
module.exports = async function ({ ethers, deployments }) {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    await deploy("MaintainerManager", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "MaintainerManager",
    });

    let MaintainerManager = await ethers.getContract("MaintainerManager");

    let MaintainerManagerProxy = await ethers.getContract("MaintainerManagerProxy");
    console.log("MaintainerManagerProxy", MaintainerManagerProxy.address);

    console.log(MaintainerManager.address);
    console.log(MaintainerManagerProxy.address);

    let ManagerProxy = await ethers.getContractAt("MaintainerManager", MaintainerManagerProxy.address);

    await ManagerProxy.upgradeTo(MaintainerManager.address);

    console.log("LightNodeUp ok");
};

module.exports.tags = ["MaintainerManagerUp"];
