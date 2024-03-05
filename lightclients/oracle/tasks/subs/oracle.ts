import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import {create,readFromFile,writeToFile,verify} from "../../utils/helper";

task("deploy:oracle", "deploy oracle")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        let salt = process.env.ORACLE_SALT || ""; 
        let Oracle = await hre.ethers.getContractFactory("Oracle");
        let param = hre.ethers.utils.defaultAbiCoder.encode(["address"], [wallet.address]);
        let result = await create(salt, Oracle.bytecode, param,hre.ethers);
        let oracle = result[0];
        console.log("oracle deploy to :",oracle);
        let d = await readFromFile(hre.network.name);
        d.oracle = oracle;
        await writeToFile(d);
        const verifyArgs = [wallet.address]
            .map((arg) => (typeof arg == "string" ? `'${arg}'` : arg))
            .join(" ");
        console.log(`To verify, run: npx hardhat verify --network ${hre.network.name} ${oracle} ${verifyArgs}`);
        // await verify(
        //     oracle,
        //     [wallet.address],
        //     "contracts/Oracle.sol:Oracle",
        //     hre.run
        // );
    });

task("oracle:setLightNode", "set light node address")
    .addParam("chainid", "chainId")
    .addParam("node","light node address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const {network} = hre;
        let d = await readFromFile(network.name);
        if(d.oracle === undefined || d.oracle === ''){
            throw("oracle not deploy")
        }
        const Oracle = await hre.ethers.getContractFactory("Oracle");

        let oracle = Oracle.attach(d.oracle);

        await (await oracle.setLightNode(taskArgs.chainid,taskArgs.node)).wait();

        let info = await oracle.lightNodeInfo(taskArgs.chainid);

        console.log(`set ${taskArgs.chainid} light node :`, info.lightNode);
 });


 task("oracle:setQuorum", "set mpt verify address")
    .addParam("chainid", "chainId")
    .addParam("quorum","quorum")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const {network} = hre;
        let d = await readFromFile(network.name);
        if(d.oracle === undefined || d.oracle === ''){
            throw("oracle not deploy")
        }
        const Oracle = await hre.ethers.getContractFactory("Oracle");

        let oracle = Oracle.attach(d.oracle);

        await (await oracle.setQuorum(taskArgs.chainid,taskArgs.quorum)).wait();

        let info = await oracle.lightNodeInfo(taskArgs.chainid);

        console.log(`set ${taskArgs.chainid} quorum :`, info.quorum);
 });


 task("oracle:updateProposer", "set mpt verify address")
 .addParam("chainid", "chainId")
 .addParam("proposers","proposers split by ,")
 .addParam("flag","statu true for add false for remove")
 .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
     let [wallet] = await hre.ethers.getSigners();
     console.log("wallet address is:", wallet.address);
     const {network} = hre;
     let d = await readFromFile(network.name);
     if(d.oracle === undefined || d.oracle === ''){
         throw("oracle not deploy")
     }
     const Oracle = await hre.ethers.getContractFactory("Oracle");
     let oracle = Oracle.attach(d.oracle);
     let proposers = taskArgs.proposers.split(",");
     await (await oracle.updateProposer(taskArgs.chainid,proposers,taskArgs.flag)).wait();
     console.log(`updateProposers ${proposers} status ${taskArgs.flag}`);
});