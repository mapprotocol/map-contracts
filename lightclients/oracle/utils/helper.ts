let fs = require("fs");
let path = require("path");

let { Wallet } = require("zksync-web3");
let { Deployer } = require("@matterlabs/hardhat-zksync-deploy");
// import { ethers,run } from "hardhat";

let DEPLOY_FACTORY = "0x6258e4d2950757A749a4d4683A7342261ce12471";
let IDeployFactory_abi = [
    "function deploy(bytes32 salt, bytes memory creationCode, uint256 value) external",
    "function getAddress(bytes32 salt) external view returns (address)",
];

export interface LightNodeInfo {
    impl: string;
    proxy: string;
}

export interface NetworkInfo {
    oracle: string;
    lightNodes: Record<string, LightNodeInfo>;
}

export interface DeployInfo {
    networks: Record<string, NetworkInfo>;
}


export async function zksyncDeploy(contractName: string, args: any[], hre: any) {
    const wallet = new Wallet(process.env.PRIVATE_KEY);
    const deployer = new Deployer(hre, wallet);
    const c_artifact = await deployer.loadArtifact(contractName);
    const c = await deployer.deploy(c_artifact, args);

    console.log(`deployed ${contractName} to ${c.address}`);
    return c.address;
}

export async function create(salt: string, bytecode: string, param: string, ethers: any) {
    let [wallet] = await ethers.getSigners();
    let factory = await ethers.getContractAt(IDeployFactory_abi, DEPLOY_FACTORY, wallet);
    let salt_hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(salt));
    console.log("deploy factory address:", factory.address);
    console.log("deploy salt:", salt);
    let addr = await factory.getAddress(salt_hash);
    console.log("deployed to :", addr);

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

export async function readFromFile(network: string) {
    let p = path.join(__dirname, "../deployments/deploy.json");
    let deploy: DeployInfo = { networks: {} };
    if (!fs.existsSync(p)) {
        deploy.networks[network] = { oracle: "", lightNodes: {} };
    } else {
        let rawdata = fs.readFileSync(p);
        deploy = JSON.parse(rawdata);
        if (!deploy.networks[network]) {
            deploy.networks[network] = { oracle: "", lightNodes: {} };
        }
    }

    return deploy;
}

export async function writeToFile(deploy: DeployInfo) {
    let p = path.join(__dirname, "../deployments/deploy.json");
    await folder("../deployments/");
    // fs.writeFileSync(p,JSON.stringify(deploy));
    fs.writeFileSync(p, JSON.stringify(deploy, null, "\t"));
}

const folder = async (reaPath: string) => {
    const absPath = path.resolve(__dirname, reaPath);
    try {
        await fs.promises.stat(absPath);
    } catch (e) {
        // {recursive: true}
        await fs.promises.mkdir(absPath, { recursive: true });
    }
};
export async function verify(addr: string, arg: any, code: string, run: any) {
    await run("verify:verify", {
        address: addr,
        constructorArguments: arg,
        contract: code,
    });
}
