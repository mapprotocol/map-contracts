const BigNumber = require("bignumber.js");
BigNumber.config({ ROUNDING_MODE: BigNumber.ROUND_FLOOR });
const initializeData = require("../deploy/config");

let {zkDeploy} = require("./utils/helper.js");
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

    let chainId = hre.network.config.chainId;

    let lightProxyAddress = taskArgs.node;
    let proxy = await ethers.getContractAt("LightNode", lightProxyAddress);

    console.log("light node proxy:", proxy.address);
    console.log("light node admin:", await proxy.getAdmin());
    console.log("light node pre impl:", await proxy.getImplementation());

    let lightNodeAddr = taskArgs.impl;
    if (lightNodeAddr === "") {
        console.log("deploy light node ...");
        if (chainId === 324 || chainId === 280) {
            lightNodeAddr = await zkDeploy("LightNode", [], hre);
        } else {
            await deploy("LightNode", {
                from: deployer.address,
                args: [],
                log: true,
                contract: "LightNode",
            });

            let lightNode = await deployments.get("LightNode");
            lightNodeAddr = lightNode.address;
        }

        await verify(lightNodeAddr, [], "contracts/LightNode.sol:LightNode", chainId, true);
    }
    console.log("deployed light node address:", lightNodeAddr);


    await (await proxy.upgradeTo(lightNodeAddr)).wait();

    console.log(`LightNode upgrade to ${await proxy.getImplementation()} successfully`);
};
