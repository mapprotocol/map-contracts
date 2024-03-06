import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import {create,readFromFile,writeToFile,verify} from "../../utils/helper";
let { deploy_contract,getTronContractAt,getDeployerAddress,toETHAddress} = require("../../utils/tron.js") ;

task("deploy:oracle", "deploy oracle")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const { deployments,network} = hre;
        let salt = process.env.ORACLE_SALT || ""; 
        let oracle = "";
        if(network.name === 'Tron' || network.name === 'TronTest'){
            let deploy_result = await deploy_contract(hre.artifacts,"Oracle",[await getDeployerAddress(network.name)],network.name);
            console.log(deploy_result);
            oracle = deploy_result[0];
        } else {
            console.log("wallet address is:", wallet.address);
            let Oracle = await hre.ethers.getContractFactory("Oracle");
            let param = hre.ethers.utils.defaultAbiCoder.encode(["address"], [wallet.address]);
            let result = await create(salt, Oracle.bytecode, param,hre.ethers);
            let oracle = result[0];
            console.log("oracle deploy to :",oracle);
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
        }
        let d = await readFromFile(hre.network.name);
        d.lightNodeInfos[network.name].oracle = oracle;
        await writeToFile(d);

    });

task("oracle:setLightNode", "set light node address")
    .addParam("chainid", "chainId")
    .addParam("node","light node address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const {network} = hre;
        let d = await readFromFile(network.name);
        if(d.lightNodeInfos[network.name].oracle === undefined || d.lightNodeInfos[network.name].oracle === ''){
            throw("oracle not deploy")
        }
        let info;
        if(network.name === 'Tron' || network.name === 'TronTest'){   
           let oracle = await getTronContractAt(hre.artifacts,"Oracle",d.lightNodeInfos[network.name].oracle,network.name);
           let n = taskArgs.node;
           if(!n.startsWith("0x")){
             n = toETHAddress(n,network.name);
           }
           let result = await oracle.setLightNode(taskArgs.chainid,n).send();
           console.log(result);
           info = await oracle.lightNodeInfo(taskArgs.chainid).call();
        } else {
            console.log("wallet address is:", wallet.address);
            const Oracle = await hre.ethers.getContractFactory("Oracle");
            let oracle = Oracle.attach(d.lightNodeInfos[network.name].oracle);
            await (await oracle.setLightNode(taskArgs.chainid,taskArgs.node)).wait();
            info = await oracle.lightNodeInfo(taskArgs.chainid);
        }
        console.log(`set ${taskArgs.chainid} light node :`, info.lightNode);

 });


 task("oracle:setQuorum", "set mpt verify address")
    .addParam("chainid", "chainId")
    .addParam("quorum","quorum")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const {network} = hre;
        let d = await readFromFile(network.name);
        if(d.lightNodeInfos[network.name].oracle === undefined || d.lightNodeInfos[network.name].oracle === ''){
            throw("oracle not deploy")
        }
        let info;
        if(network.name === 'Tron' || network.name === 'TronTest'){   
            let oracle = await getTronContractAt(hre.artifacts,"Oracle",d.lightNodeInfos[network.name].oracle,network.name);
            let result = await oracle.setQuorum(taskArgs.chainid,taskArgs.quorum).send();
            console.log(result);
            info = await oracle.lightNodeInfo(taskArgs.chainid).call();
         } else {
            console.log("wallet address is:", wallet.address);
            const Oracle = await hre.ethers.getContractFactory("Oracle");
            let oracle = Oracle.attach(d.oracle);
            await (await oracle.setQuorum(taskArgs.chainid,taskArgs.quorum)).wait();
            let info = await oracle.lightNodeInfo(taskArgs.chainid);
         }
       
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
     if(d.lightNodeInfos[network.name].oracle === undefined || d.lightNodeInfos[network.name].oracle === ''){
        throw("oracle not deploy")
    }
     let proposers = taskArgs.proposers.split(",");
     if(network.name === 'Tron' || network.name === 'TronTest'){   
        let oracle = await getTronContractAt(hre.artifacts,"Oracle",d.lightNodeInfos[network.name].oracle,network.name);
        let p :Array<string> = new Array(proposers.length);
        for(let i = 0; i < proposers.length; i ++){
            if(!proposers[i].startsWith("0x")){
               p[i] = await toETHAddress(proposers[i],network.name)
            } else {
                p[i] = proposers[i]
            }
        }
        let result = await oracle.updateProposer(taskArgs.chainid,p,taskArgs.flag).send();
        console.log(result);
     } else { 
        const Oracle = await hre.ethers.getContractFactory("Oracle");
        let oracle = Oracle.attach(d.oracle);
        await (await oracle.updateProposer(taskArgs.chainid,proposers,taskArgs.flag)).wait();
     }
     console.log(`updateProposers ${proposers} status ${taskArgs.flag}`);
});