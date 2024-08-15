const {zkDeploy, create} = require("./utils/helper");

let { verify } = require("./utils/verify.js");

let IDeployFactory_abi = [
    "function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external",
    "function getAddress(bytes32 salt) external view returns (address)",
];

module.exports = async (taskArgs, hre) => {
    const {deploy} = hre.deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    let chainId = hre.network.config.chainId;


    let verifierAddr;

    if (chainId === 324 || chainId === 280) {
        // zksync mainnet or testnet
        verifierAddr = await zkDeploy("VerifyTool", [], hre);
    } else if (taskArgs.salt === "")  {
        await deploy("VerifyTool", {
            from: deployer.address,
            args: [],
            log: true,
            contract: "VerifyTool",
        });

        let verifyTool = await deployments.get("VerifyTool");
        verifierAddr = verifyTool.address;
    } else {
        console.log("verifyTool salt:", taskArgs.salt);
        let VerifyTool = await ethers.getContractFactory("VerifyTool");
        let params = ethers.utils.defaultAbiCoder.encode([], []);
        let createResult = await create(taskArgs.salt, VerifyTool.bytecode, params);
        if (!createResult[1]) {
            return;
        }
        verifierAddr = createResult[0];
    }

    await verify(verifierAddr, [], "contracts/VerifyTool.sol:VerifyTool", chainId, true);

    console.log(`VerifyTool  contract address is ${verifierAddr}`);
};
