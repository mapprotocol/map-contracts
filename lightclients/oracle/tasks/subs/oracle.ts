import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { create, readFromFile, writeToFile, verify } from "../../utils/helper";
let { deploy_contract, getTronContractAt, getDeployerAddress, toETHAddress } = require("../../utils/tron.js");

task("oracle:deploy", "deploy oracle")
    .addOptionalParam("salt", "oracle salt", "", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { deployments, network } = hre;
        const { deploy } = deployments;

        let salt = taskArgs.salt;
        let oracle = "";
        if (network.name === "Tron" || network.name === "TronTest") {
            let deploy_result = await deploy_contract(
                hre.artifacts,
                "Oracle",
                [await getDeployerAddress(network.name)],
                network.name
            );
            console.log(deploy_result);
            oracle = deploy_result[0];
        } else if (salt === "") {
            let result = await deploy("Oracle", {
                from: wallet.address,
                args: [wallet.address],
                log: true,
                contract: "Oracle",
            });
            oracle = result.address;
        } else {
            console.log("wallet address is:", wallet.address);
            let Oracle = await hre.ethers.getContractFactory("Oracle");
            let param = hre.ethers.utils.defaultAbiCoder.encode(["address"], [wallet.address]);
            let result = await create(salt, Oracle.bytecode, param, hre.ethers);
            let oracle = result[0];
        }
        console.log("oracle deploy to :", oracle);
        const verifyArgs = [wallet.address].map((arg) => (typeof arg == "string" ? `'${arg}'` : arg)).join(" ");
        console.log(`To verify, run: npx hardhat verify --network ${hre.network.name} ${oracle} ${verifyArgs}`);
        // await verify(
        //     oracle,
        //     [wallet.address],
        //     "contracts/Oracle.sol:Oracle",
        //     hre.run
        // );

        let d = await readFromFile(hre.network.name);
        d.networks[network.name].oracle = oracle;
        await writeToFile(d);
    });

task("oracle:setLightNode", "set light node address")
    .addOptionalParam("chain", "chainId", 0, types.int)
    .addOptionalParam("node", "light node address", "node", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { network } = hre;
        let d = await readFromFile(network.name);
        if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
            throw "oracle not deploy";
        }
        console.log("oracle:", d.networks[network.name].oracle);

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
        console.log("light node:", node);

        let info;
        if (network.name === "Tron" || network.name === "TronTest") {
            let oracle = await getTronContractAt(
                hre.artifacts,
                "Oracle",
                d.networks[network.name].oracle,
                network.name
            );
            let n = node;
            if (!n.startsWith("0x")) {
                n = toETHAddress(n, network.name);
            }
            let result = await oracle.setLightNode(chain, n).send();
            console.log(result);
            info = await oracle.lightNodeInfo(chain).call();
        } else {
            console.log("wallet address is:", wallet.address);
            const Oracle = await hre.ethers.getContractFactory("Oracle");
            let oracle = Oracle.attach(d.networks[network.name].oracle);
            await (await oracle.setLightNode(chain, node)).wait();
            info = await oracle.lightNodeInfo(chain);
        }
        console.log(`set ${chain} light node :`, info.lightNode);
    });

task("oracle:setQuorum", "set mpt verify address")
    .addOptionalParam("chain", "chainId", 0, types.int)
    .addParam("quorum", "quorum")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { network } = hre;
        let d = await readFromFile(network.name);
        if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
            throw "oracle not deploy";
        }
        let chain = taskArgs.chain;
        if (chain == 0) {
            chain = Object.keys(d.networks[network.name].lightNodes)[0];
        }
        let info;
        if (network.name === "Tron" || network.name === "TronTest") {
            let oracle = await getTronContractAt(
                hre.artifacts,
                "Oracle",
                d.networks[network.name].oracle,
                network.name
            );
            let result = await oracle.setQuorum(chain, taskArgs.quorum).send();
            console.log(result);
            info = await oracle.lightNodeInfo(chain).call();
        } else {
            console.log("wallet address is:", wallet.address);
            const Oracle = await hre.ethers.getContractFactory("Oracle");
            let oracle = Oracle.attach(d.networks[network.name].oracle);
            await (await oracle.setQuorum(chain, taskArgs.quorum)).wait();
            info = await oracle.lightNodeInfo(chain);
        }

        console.log(`set ${chain} quorum :`, info.quorum);
    });

task("oracle:updateProposer", "update proposer address")
    .addOptionalParam("chain", "chainId", 0, types.int)
    .addParam("proposers", "proposers split by ,")
    .addOptionalParam("flag", "statu true for add false for remove", true, types.boolean)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const { network } = hre;
        let d = await readFromFile(network.name);
        if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
            throw "oracle not deploy";
        }
        let chain = taskArgs.chain;
        if (chain == 0) {
            chain = Object.keys(d.networks[network.name].lightNodes)[0];
        }
        let proposers = taskArgs.proposers.split(",");
        if (network.name === "Tron" || network.name === "TronTest") {
            let oracle = await getTronContractAt(
                hre.artifacts,
                "Oracle",
                d.networks[network.name].oracle,
                network.name
            );
            let p: Array<string> = new Array(proposers.length);
            for (let i = 0; i < proposers.length; i++) {
                if (!proposers[i].startsWith("0x")) {
                    p[i] = await toETHAddress(proposers[i], network.name);
                } else {
                    p[i] = proposers[i];
                }
            }
            let result = await oracle.updateProposer(chain, p, taskArgs.flag).send();
            console.log(result);
        } else {
            const Oracle = await hre.ethers.getContractFactory("Oracle");
            let oracle = Oracle.attach(d.networks[network.name].oracle);
            await (await oracle.updateProposer(chain, proposers, taskArgs.flag)).wait();
        }
        console.log(`updateProposers ${proposers} status ${taskArgs.flag}`);
    });

task("oracle:pause", "update proposer address").setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    let [wallet] = await hre.ethers.getSigners();
    console.log("wallet address is:", wallet.address);
    const { network } = hre;
    let d = await readFromFile(network.name);
    if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
        throw "oracle not deploy";
    }
    console.log("oracle address is:", d.networks[network.name].oracle);
    if (network.name === "Tron" || network.name === "TronTest") {
        let oracle = await getTronContractAt(hre.artifacts, "Oracle", d.networks[network.name].oracle, network.name);
        let result = await oracle.togglePause(true).send();
        console.log(result);
    } else {
        const Oracle = await hre.ethers.getContractFactory("Oracle");
        let oracle = Oracle.attach(d.networks[network.name].oracle);
        await (await oracle.togglePause(true)).wait();
    }
    console.log(`oracle pause`);
});

task("oracle:unpause", "update proposer address").setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
    let [wallet] = await hre.ethers.getSigners();
    console.log("wallet address is:", wallet.address);
    const { network } = hre;
    let d = await readFromFile(network.name);
    if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
        throw "oracle not deploy";
    }
    console.log("oracle address is:", d.networks[network.name].oracle);
    if (network.name === "Tron" || network.name === "TronTest") {
        let oracle = await getTronContractAt(hre.artifacts, "Oracle", d.networks[network.name].oracle, network.name);
        let result = await oracle.togglePause(false).send();
        console.log(result);
    } else {
        const Oracle = await hre.ethers.getContractFactory("Oracle");
        let oracle = Oracle.attach(d.networks[network.name].oracle);
        await (await oracle.togglePause(false)).wait();
    }
    console.log(`oracle unpause`);
});

task("oracle:getNode", "get light node info")
    .addOptionalParam("chain", "chainId", 0, types.int)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const { network } = hre;
        let d = await readFromFile(network.name);
        if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
            throw "oracle not deploy";
        }
        let chain = taskArgs.chain;
        if (chain == 0) {
            chain = Object.keys(d.networks[network.name].lightNodes)[0];
        }
        let info;
        if (network.name === "Tron" || network.name === "TronTest") {
            let oracle = await getTronContractAt(
                hre.artifacts,
                "Oracle",
                d.networks[network.name].oracle,
                network.name
            );
            let result = await oracle.setQuorum(chain, taskArgs.quorum).send();
            console.log(result);
            info = await oracle.lightNodeInfo(chain).call();
        } else {
            const Oracle = await hre.ethers.getContractFactory("Oracle");
            let oracle = Oracle.attach(d.networks[network.name].oracle);
            info = await oracle.lightNodeInfo(chain);
        }
        console.log(`node ${chain} ${info.lightNode} quorum: ${info.quorum}, proposers: ${info.proposerCount}`);
    });
