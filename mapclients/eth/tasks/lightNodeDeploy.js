const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });
const initializeData = require("../deploy/config");


let IDeployFactory_abi = [
    "function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external",
    "function getAddress(bytes32 salt) external view returns (address)",
];

module.exports = async (taskArgs, hre) => {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let verifier = taskArgs.verify;
    if (verifier === "") {
        await deploy("VerifyTool", {
            from: deployer.address,
            args: [],
            log: true,
            contract: "VerifyTool",
        });

        let verifyTool = await deployments.get("VerifyTool");
        verifier = verifyTool.address;
    }
    console.log("verify tool addr", verifier);

    await deploy("LightNode", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "LightNode",
    });

    let LightNode = await deployments.get("LightNode");
    let lightNode = await ethers.getContractAt("LightNode", LightNode.address);

    console.log(lightNode.address);
    //let validatorNum = initializeData.initData.validators;
    let validatorNum = initializeData.validators;
    let g1List = [];
    let addresss = [];
    let weights = [];
    for (let i = 0; i < validatorNum.length; i++) {
        let temp = [validatorNum[i].g1_pub_key.x, validatorNum[i].g1_pub_key.y];
        g1List.push(temp);
        addresss.push(validatorNum[i].address);
        weights.push(validatorNum[i].weight);
    }

    let threshold = initializeData.threshold;

    let epoch = initializeData.epoch;

    let epochSize = initializeData.epoch_size;

    let data = lightNode.interface.encodeFunctionData("initialize", [
        threshold,
        addresss,
        g1List,
        weights,
        epoch,
        epochSize,
        verifier
    ]);
    console.log("initialize success");

    let lightProxy = await ethers.getContractFactory("LightNodeProxy");

    let initData = await ethers.utils.defaultAbiCoder.encode(["address", "bytes"], [lightNode.address, data]);

    let deployData = lightProxy.bytecode + initData.substring(2);

    console.log("light node salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let factory = await ethers.getContractAt(IDeployFactory_abi, taskArgs.factory);
    //let factory = await ethers.getContractAt("IDeployFactory", taskArgs.factory);

    console.log("deploy factory address:", factory.address);

    await (await factory.connect(deployer).deploy(hash, deployData, 0)).wait();

    let lightProxyAddress = await factory.connect(deployer).getAddress(hash);

    console.log("deployed light node proxy address:", lightProxyAddress);

    let proxy = await ethers.getContractAt("LightNode", lightProxyAddress);

    let owner = await proxy.connect(deployer).getAdmin();

    console.log(
        `LightNode Proxy contract address is ${lightProxyAddress}, init admin address is ${owner}, deploy contract salt is ${hash}`
    );
};
