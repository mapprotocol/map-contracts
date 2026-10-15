import { task, types } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { create, getCreateAddress, readFromFile, writeToFile, verifyWithFallback } from "../../utils/helper";
import { getSigInfo, compare } from "../MultsigUtils";

task("nodeV3:deploy", "deploy oracle light node v3")
    .addOptionalParam("salt", "oracle salt", "", types.string)
    .addParam("chain", "chain id")
    .addOptionalParam("impl", "impl address", "", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const { deployments, network } = hre;
        const { deploy } = deployments;
        let [wallet] = await hre.ethers.getSigners();

        let salt = taskArgs.salt;
        let impl = taskArgs.impl;
        let LightNode = await hre.ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");

        let node;
        let proxyDeployed = true;
        console.log("wallet address is:", wallet.address);
        if (salt !== "") {
            const created = await getCreateAddress(salt, hre.ethers);
            node = created.address;
            const proxyExists = (await hre.ethers.provider.getCode(node)) !== "0x";
            if (proxyExists) {
                proxyDeployed = false;
            }
        }
        if (impl === "") {
            if (proxyDeployed) {
                let implDeploy = await deploy(`LightNodeV3_${taskArgs.chain}`, {
                    from: wallet.address,
                    args: [],
                    log: true,
                    contract: "contracts/v3/LightNodeV3.sol:LightNodeV3",
                });
                impl = implDeploy.address;
            } else {
                impl = "";
            }
        }
        console.log("impl address :", impl);

        let implParam = LightNode.interface.encodeFunctionData("initialize", [taskArgs.chain, wallet.address]);
        if (salt === "") {
            let result = await deploy(`LightNodeProxy_${taskArgs.chain}`, {
                from: wallet.address,
                args: [impl, implParam],
                log: true,
                contract: "LightNodeProxy",
            });
            node = result.address;
            proxyDeployed = result.newlyDeployed;
        } else if (proxyDeployed) {
            let param = hre.ethers.utils.defaultAbiCoder.encode(["address", "bytes"], [impl, implParam]);
            let LightNodeProxy = await hre.ethers.getContractFactory("LightNodeProxy");
            let result = await create(salt, LightNodeProxy.bytecode, param, hre.ethers);
            node = result[0];
            proxyDeployed = result[1];
        }

        console.log("node address :", node);
        let d = await readFromFile(network.name);
        if (!d.networks[network.name].lightNodes[taskArgs.chain]) {
            d.networks[network.name].lightNodes[taskArgs.chain] = { proxy: "", impl: "" };
        }

        d.networks[network.name].lightNodes[taskArgs.chain].proxy = node;
        if (impl !== "" && proxyDeployed) {
            d.networks[network.name].lightNodes[taskArgs.chain].impl = impl;
        }
        await writeToFile(d);

        if (impl !== "" && proxyDeployed) {
            try {
                await verifyWithFallback(impl, [], "contracts/v3/LightNodeV3.sol:LightNodeV3", hre);
                console.log("verified lightnode impl:", impl);
            } catch (error) {
                console.log("verify lightnode impl failed:", error);
            }
        }

        if (proxyDeployed) {
            try {
                await verifyWithFallback(node, [impl, implParam], "contracts/LightNodeProxy.sol:LightNodeProxy", hre);
                console.log("verified lightnode proxy:", node);
            } catch (error) {
                console.log("verify lightnode proxy failed:", error);
            }
        } else {
            console.log("skip lightnode proxy deploy and verify: address already had code, please change salt if you want a new light node proxy");
        }
    });

task("nodeV3:upgrade", "upgrade oracle light node v3")
    .addOptionalParam("chain", "chainId", 0, types.int)
    .addOptionalParam("node", "light node address", "node", types.string)
    .addOptionalParam("impl", "impl address", "impl", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const { deployments, network } = hre;
        const { deploy } = deployments;

        let d = await readFromFile(network.name);
        let chain = taskArgs.chain;
        if (chain == 0) {
            chain = Object.keys(d.networks[network.name].lightNodes)[0];
        }

        let impl = taskArgs.impl;
        let node = taskArgs.node;

        if (node === "node") {
            if (!d.networks[network.name].lightNodes[chain]) {
                throw "oracle light node not deploy";
            }
            if (
                d.networks[network.name].lightNodes[chain].proxy === undefined ||
                d.networks[network.name].lightNodes[chain].proxy === ""
            ) {
                throw "oracle light node not deploy";
            }
            node = d.networks[network.name].lightNodes[chain].proxy;
        }
        console.log("light node proxy: ", node);

        if (impl === "impl") {
            let deployed = await deploy("LightNodeV3", {
                from: wallet.address,
                args: [],
                log: true,
                contract: "contracts/v3/LightNodeV3.sol:LightNodeV3",
            });
            impl = deployed.address;

            try {
                await verifyWithFallback(impl, [], "contracts/v3/LightNodeV3.sol:LightNodeV3", hre);
                console.log("verified lightnode impl:", impl);
            } catch (error) {
                console.log("verify lightnode impl failed:", error);
            }
        }

        const LightNode = await hre.ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
        let proxy = LightNode.attach(node);
        console.log("old impl :", await proxy.getImplementation());
        await (await proxy.upgradeTo(impl)).wait();
        console.log("new impl :", await proxy.getImplementation());

        d.networks[network.name].lightNodes[chain].impl = impl;
        await writeToFile(d);
    });

task("nodeV3:updateMultisig", "update multi sign address for light node v3")
    .addOptionalParam("chain", "chainId", 0, types.int)
    .addOptionalParam("node", "light node address", "node", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { network } = hre;

        let d = await readFromFile(network.name);
        let chain = taskArgs.chain;
        if (chain == 0) {
            chain = Object.keys(d.networks[network.name].lightNodes)[0];
        }
        console.log("light node chain id:", chain);

        let node = taskArgs.node;
        if (node === "node") {
            if (!d.networks[network.name].lightNodes[chain]) {
                throw "oracle light node not deploy";
            }
            if (
                d.networks[network.name].lightNodes[chain].proxy === undefined ||
                d.networks[network.name].lightNodes[chain].proxy === ""
            ) {
                throw "oracle light node not deploy";
            }
            node = d.networks[network.name].lightNodes[chain].proxy;
        }
        console.log("light node address:", node);

        const LightNode = await hre.ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
        let proxy = LightNode.attach(node);

        let oldInfo = await proxy.multisigInfo();
        console.log("oldInfo :", oldInfo);
        let sig = getSigInfo();
        let same = await compare(oldInfo.version, sig);
        if (same) {
            console.log("Multisg already set");
        } else {
            await (await proxy.updateMultisig(sig.quorum, sig.signers)).wait();
        }
    });

task("nodeV3:grantRole", "grant or revoke role for light node v3")
    .addParam("role", "role name: upgrade | manager | pauser")
    .addParam("account", "account address")
    .addOptionalParam("grant", "grant or revoke", true, types.boolean)
    .addOptionalParam("chain", "chainId", 0, types.int)
    .addOptionalParam("node", "light node address", "node", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { network } = hre;

        let d = await readFromFile(network.name);
        let chain = taskArgs.chain;
        if (chain == 0) {
            chain = Object.keys(d.networks[network.name].lightNodes)[0];
        }

        let node = taskArgs.node;
        if (node === "node") {
            if (!d.networks[network.name].lightNodes[chain]) {
                throw "oracle light node not deploy";
            }
            if (
                d.networks[network.name].lightNodes[chain].proxy === undefined ||
                d.networks[network.name].lightNodes[chain].proxy === ""
            ) {
                throw "oracle light node not deploy";
            }
            node = d.networks[network.name].lightNodes[chain].proxy;
        }

        console.log("light node address:", node);
        console.log("wallet address is:", wallet.address);

        const LightNode = await hre.ethers.getContractFactory("contracts/v3/LightNodeV3.sol:LightNodeV3");
        let lightNode = LightNode.attach(node);

        let role;
        if (taskArgs.role === "upgrade" || taskArgs.role === "upgrader") {
            role = await lightNode.UPGRADER_ROLE();
        } else if (taskArgs.role === "manage" || taskArgs.role === "manager") {
            role = await lightNode.MANAGER_ROLE();
        } else if (taskArgs.role === "pause" || taskArgs.role === "pauser") {
            role = await lightNode.PAUSER_ROLE();
        } else {
            role = hre.ethers.constants.HashZero;
        }

        if (taskArgs.grant) {
            await (await lightNode.grantRole(role, taskArgs.account)).wait();
            console.log(`grant ${taskArgs.account} role ${role}`);
        } else {
            await (await lightNode.revokeRole(role, taskArgs.account)).wait();
            console.log(`revoke ${taskArgs.account} role ${role}`);
        }
    });
