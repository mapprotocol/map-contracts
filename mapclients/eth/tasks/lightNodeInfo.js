const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });
const initializeData = require("../deploy/config");

let { zkDeploy } = require("./utils/helper.js");
let { verify } = require("./utils/verify.js");

let IDeployFactory_abi = [
    "function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external",
    "function getAddress(bytes32 salt) external view returns (address)",
];

module.exports = async (taskArgs, hre) => {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    let lightProxyAddress = taskArgs.node;
    let proxy = await ethers.getContractAt("LightNode", lightProxyAddress);

    console.log("light node proxy:", proxy.address);
    console.log("light node admin:", await proxy.getAdmin());
    console.log("light node pre impl:", await proxy.getImplementation());

    console.log("epoch size:", await proxy.epochSize());
    console.log("maxEpochs:", await proxy.maxEpochs());
    console.log("startHeight:", await proxy.startHeight());
    console.log("headerHeight:", await proxy.headerHeight());

    let epoch = await proxy.getEpoch(taskArgs.id);
    console.log("epoch", epoch[0]);
    console.log("number", epoch[1]);
    console.log("aggX", epoch[2]);
    console.log("aggY", epoch[3]);
};
