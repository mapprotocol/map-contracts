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
    console.log(`zksync deploy ${contractName} ...`);
    const wallet = new Wallet(process.env.PRIVATE_KEY);
    const deployer = new Deployer(hre, wallet);
    const c_artifact = await deployer.loadArtifact(contractName);
    const c = await deployer.deploy(c_artifact, args);

    console.log(`deployed ${contractName} to ${c.address}`);
    return c.address;
}

export async function create(salt: string, bytecode: string, param: string, ethers: any) {
    let { factory, saltHash: salt_hash, address: addr } = await getCreateAddress(salt, ethers);
    console.log("deploy factory address:", factory.address);
    console.log("deploy salt:", salt);
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

export async function getCreateAddress(salt: string, ethers: any) {
    let [wallet] = await ethers.getSigners();
    let factory = await ethers.getContractAt(IDeployFactory_abi, DEPLOY_FACTORY, wallet);
    let saltHash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(salt));
    let address = await factory.getAddress(saltHash);
    return { factory, saltHash, address };
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

export async function verifyWithFallback(addr: string, arg: any, code: string, hre: any) {
    try {
        await verify(addr, arg, code, hre.run);
        return;
    } catch (error: any) {
        const message = String(error?.message || error);
        const networkError =
            message.includes("Failed to obtain list of solc versions") ||
            message.includes("secure TLS connection was established") ||
            message.includes("Client network socket disconnected");
        const inferContractError =
            message.includes("bytecode doesn't match any of your local contracts") ||
            message.includes("bytecode doesn't match");

        if (!networkError && !inferContractError) {
            throw error;
        }

        const apiURL = getApiUrl(hre);
        const buildInfo = await getBuildInfo(code);
        const contractInfo = buildInfo.output.contracts[code.split(":")[0]][code.split(":")[1]];
        const constructorArguments = encodeConstructorArgs(contractInfo.abi, arg, hre.ethers);
        const optimizationEnabled = !!buildInfo.input.settings?.optimizer?.enabled;
        const runs = buildInfo.input.settings?.optimizer?.runs ?? 200;
        const solcFullVersion = `v${buildInfo.solcLongVersion}`;

        const body = new URLSearchParams({
            module: "contract",
            action: "verifysourcecode",
            contractaddress: addr,
            sourceCode: JSON.stringify(buildInfo.input),
            codeformat: "solidity-standard-json-input",
            contractname: code,
            compilerversion: solcFullVersion,
            optimizationUsed: optimizationEnabled ? "1" : "0",
            runs: String(runs),
            constructorArguements: constructorArguments,
            apikey: getApiKey(hre),
        });

        const response = await fetch(apiURL, {
            method: "POST",
            headers: { "content-type": "application/x-www-form-urlencoded" },
            body,
        });
        const result = await response.json();
        if (isAlreadyVerified(result)) {
            return;
        }
        if (result.status !== "1") {
            throw new Error(`verify submit failed: ${result.result || result.message}`);
        }

        const guid = result.result;
        for (let i = 0; i < 10; i++) {
            await new Promise((resolve) => setTimeout(resolve, 3000));
            const pollUrl = `${apiURL}?module=contract&action=checkverifystatus&guid=${guid}&apikey=${encodeURIComponent(
                getApiKey(hre)
            )}`;
            const pollResponse = await fetch(pollUrl);
            const pollResult = await pollResponse.json();
            if (isAlreadyVerified(pollResult)) {
                return;
            }
            if (pollResult.status === "1") {
                return;
            }
            if (
                typeof pollResult.result === "string" &&
                !pollResult.result.includes("Pending in queue") &&
                !pollResult.result.includes("In Progress")
            ) {
                throw new Error(`verify status failed: ${pollResult.result}`);
            }
        }

        throw new Error("verify status timeout");
    }
}

function isAlreadyVerified(result: any): boolean {
    const message = `${result?.message || ""} ${result?.result || ""}`.toLowerCase();
    return message.includes("already verified");
}

function getApiKey(hre: any): string {
    const apiKey = hre.config.etherscan?.apiKey;
    if (typeof apiKey === "string") {
        return apiKey.trim();
    }
    if (apiKey && typeof apiKey === "object") {
        return String(apiKey[hre.network.name] || "").trim();
    }
    return "";
}

function getApiUrl(hre: any): string {
    const customChains = hre.config.etherscan?.customChains || [];
    const custom = customChains.find((c: any) => c.network === hre.network.name);
    if (!custom?.urls?.apiURL) {
        throw new Error(`no etherscan customChains apiURL configured for ${hre.network.name}`);
    }
    return custom.urls.apiURL;
}

async function getBuildInfo(contractFqn: string) {
    const [sourceName, contractName] = contractFqn.split(":");
    const buildInfoDir = path.join(__dirname, "../artifacts/build-info");
    const files = fs.readdirSync(buildInfoDir);
    for (const file of files) {
        const fullPath = path.join(buildInfoDir, file);
        const buildInfo = JSON.parse(fs.readFileSync(fullPath, "utf8"));
        if (buildInfo?.output?.contracts?.[sourceName]?.[contractName]) {
            return buildInfo;
        }
    }
    throw new Error(`build-info not found for ${contractFqn}`);
}

function encodeConstructorArgs(abi: any[], args: any, ethers: any): string {
    if (typeof args === "string" && args.startsWith("0x")) {
        return args.replace(/^0x/, "");
    }
    const ctor = abi.find((item) => item.type === "constructor");
    if (!ctor || !ctor.inputs || ctor.inputs.length === 0) {
        return "";
    }
    const values = Array.isArray(args) ? args : [];
    const types = ctor.inputs.map((input: any) => input.type);
    return ethers.utils.defaultAbiCoder.encode(types, values).replace(/^0x/, "");
}
