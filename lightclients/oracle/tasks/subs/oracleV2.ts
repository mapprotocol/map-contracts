import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { getSigInfo, compare, Multisig} from "../MultsigUtils"
import { create, readFromFile, writeToFile, zksyncDeploy, verify } from "../../utils/helper";
let { deploy_contract, getTronContractAt, getDeployerAddress, toETHAddress } = require("../../utils/tron.js");

task("oracleV2:deploy", "deploy oracle")
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
                "OracleV2",
                [await getDeployerAddress(network.name)],
                network.name
            );
            console.log(deploy_result);
            oracle = deploy_result[0];
        } else if (network.config.chainId === 324 || network.config.chainId === 280) {
            oracle = await zksyncDeploy("OracleV2", [wallet.address], hre);
        } else if (salt === "") {
            let result = await deploy("OracleV2", {
                from: wallet.address,
                args: [wallet.address],
                log: true,
                contract: "OracleV2",
            });
            oracle = result.address;
        } else {
            console.log("wallet address is:", wallet.address);
            let Oracle = await hre.ethers.getContractFactory("OracleV2");
            let param = hre.ethers.utils.defaultAbiCoder.encode(["address"], [wallet.address]);
            let result = await create(salt, Oracle.bytecode, param, hre.ethers);
            oracle = result[0];
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

task("OracleV2:updateMultisg", "set light node address")
    .addOptionalParam("oracle", "oracle address", "", types.string)
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { network } = hre;
        let d = await readFromFile(network.name);

        let oracleAddr = taskArgs.oracle;
        if (oracleAddr === "") {
            if (d.networks[network.name].oracle === undefined || d.networks[network.name].oracle === "") {
                throw "oracle not deploy";
            }
            oracleAddr = d.networks[network.name].oracle;
        }
        console.log("oracle manager address:", oracleAddr);

        if (network.name === "Tron" || network.name === "TronTest") {
            let oracle = await getTronContractAt(
                hre.artifacts,
                "OracleV2",
                oracleAddr,
                network.name
            );
            let old_info = await oracle.multisigInfo().call();
            console.log("old_info :", old_info);
            
            let sig = getSigInfo();

            let d = await compare(old_info.version,sig);
            if(d) {
                console.log("Multisg already set");
            } else {
                let result = await oracle.updateMultisg(sig.quorum,sig.signers).send();
                console.log("updateMultisg: ", result); 
            }
        } else {
            console.log("wallet address is:", wallet.address);
            const Oracle = await hre.ethers.getContractFactory("OracleV2");
            let oracle = Oracle.attach(oracleAddr);
            let old_info = await oracle.multisigInfo().call();
            console.log("old_info :", old_info);
            let sig = getSigInfo();
            let d = await compare(old_info.version,sig);
            if(d) {
                console.log("Multisg already set");
            } else {
                await(await oracle.updateMultisg(sig.quorum,sig.signers)).wait();
            }
        }
    });

