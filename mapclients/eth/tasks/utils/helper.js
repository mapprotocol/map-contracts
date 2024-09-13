let fs = require("fs");
let path = require("path");
const { Wallet } = require("zksync-web3");
const { Deployer } = require("@matterlabs/hardhat-zksync-deploy");

DEPLOY_FACTORY = "0x6258e4d2950757A749a4d4683A7342261ce12471";
let IDeployFactory_abi = [
    "function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external",
    "function getAddress(bytes32 salt) external view returns (address)",
];
async function create(salt, bytecode, param) {
    let [wallet] = await ethers.getSigners();
    let factory = await ethers.getContractAt(IDeployFactory_abi, DEPLOY_FACTORY, wallet);
    let salt_hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(salt));
    console.log("deploy factory address:", factory.address);
    console.log("deploy salt:", salt);
    let addr = await factory.getAddress(salt_hash);
    console.log(`deploy to ${addr} ...`);

    let code = await ethers.provider.getCode(addr);
    let redeploy = false;
    if (code === "0x") {
        let create_code = ethers.utils.solidityPack(["bytes", "bytes"], [bytecode, param]);
        let create = await (await factory.deploy(salt_hash, create_code, 0)).wait();
        if (create.status == 1) {
            console.log("deployed to :", addr);
            redeploy = true;
        } else {
            console.log("deploy fail");
            throw "deploy fail";
        }
    } else {
        console.log("already deploy, please change the salt if if want to deploy another contract ...");
    }

    return [addr, redeploy];
}

async function zkDeploy(contractName, args, hre) {
    const wallet = new Wallet(process.env.PRIVATE_KEY);
    const deployer = new Deployer(hre, wallet);
    const c_artifact = await deployer.loadArtifact(contractName);
    const c = await deployer.deploy(c_artifact, args);
    return c.address;
}

module.exports = {
    create,
    zkDeploy,
};
