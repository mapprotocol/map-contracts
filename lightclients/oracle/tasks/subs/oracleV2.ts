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
        console.log("wallet address is:", wallet.address);
        let Oracle = await hre.ethers.getContractFactory("OracleV2");
        let param = hre.ethers.utils.defaultAbiCoder.encode(["address"], [wallet.address]);
        let result = await create(salt, Oracle.bytecode, param, hre.ethers);
        let oracle = result[0];
        console.log("oracle deploy to :", oracle);
        const verifyArgs = [wallet.address].map((arg) => (typeof arg == "string" ? `'${arg}'` : arg)).join(" ");
        console.log(`To verify, run: npx hardhat verify --network ${hre.network.name} ${oracle} ${verifyArgs}`);
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
        console.log("wallet address is:", wallet.address);
        const Oracle = await hre.ethers.getContractFactory("OracleV2");
        let oracle = Oracle.attach(oracleAddr);
        let old_info = await oracle.multisigInfo();
        console.log("old_info :", old_info);
        let sig = getSigInfo();
        let c = await compare(old_info.version,sig);
        if(c) {
            console.log("Multisg already set");
        } else {
            await(await oracle.updateMultisg(sig.quorum,sig.signers)).wait();
        }

    });

