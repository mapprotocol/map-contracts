const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });
const initializeDataMainnet = require("../deploy/config.mainnet");
const initializeDataTest = require("../deploy/config.testnet");

module.exports = async (taskArgs, hre) => {
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

    let LightNode = await deployments.get("LightNode");
    let lightNode = await ethers.getContractAt("LightNode", LightNode.address);

    console.log(lightNode.address);

    let validatorsInfo;

    let validatorsCount;

    let epoch;

    let epochSize;

    if (taskArgs.chain === "mainnet") {
        validatorsInfo = initializeDataMainnet.validatorsInfo;

        validatorsCount = initializeDataMainnet.validatorsCount;

        epoch = initializeDataMainnet.epoch;

        epochSize = initializeDataMainnet.epochSize;
    } else if (taskArgs.chain === "test") {
        validatorsInfo = initializeDataTest.validatorsInfo;

        validatorsCount = initializeDataTest.validatorsCount;

        epoch = initializeDataTest.epoch;

        epochSize = initializeDataTest.epochSize;
    }

    let data = lightNode.interface.encodeFunctionData("initialize", [
        validatorsInfo,
        validatorsCount,
        epoch,
        epochSize,
        taskArgs.verifytool,
        taskArgs.verifier,
    ]);

    let lightProxy = await ethers.getContractFactory("LightNodeProxy");

    let initData = await ethers.utils.defaultAbiCoder.encode(["address", "bytes"], [lightNode.address, data]);

    let deployData = lightProxy.bytecode + initData.substring(2);

    console.log("light node salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let factory = await ethers.getContractAt("IDeployFactory", taskArgs.factory);

    console.log("deploy factory address:", factory.address);

    await (await factory.connect(deployer).deploy(hash, deployData, 0, { gasLimit: "10000000" })).wait();

    let lightProxyAddress = await factory.connect(deployer).getAddress(hash);

    console.log("deployed light node proxy address:", lightProxyAddress);

    let proxy = await ethers.getContractAt("LightNode", lightProxyAddress);

    let owner = await proxy.connect(deployer).getAdmin();

    console.log(
        `LightNode Proxy contract address is ${lightProxyAddress}, init admin address is ${owner}, deploy contract salt is ${hash}`
    );
};
