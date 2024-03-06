import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
// import { DeployFunction } from "hardhat-deploy/types";
import {create,readFromFile,writeToFile,verify} from "../../utils/helper";
let { deploy_contract,getTronContractAt,getDeployerAddress,toETHAddress} = require("../../utils/tron.js") ;


task("deploy:node", "deploy oracle light node")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        const { deployments,network} = hre;
        const { deploy } = deployments;
        let [wallet] = await hre.ethers.getSigners();
        let salt = process.env.NODE_SALT || "";  
        let chainId = process.env.CHAIN_Id; 
        let nodeType = process.env.NODE_TYPE
        let mpt = process.env.MPT_VERIFY;
        let LightNode = await hre.ethers.getContractFactory("LightNode");
        
        let node;
        let impl = '';
        if(network.name === 'Tron' || network.name === 'TronTest'){
            if(mpt === undefined || mpt === ''){
                let result = await deploy_contract(hre.artifacts,"MPTVerify",[],network.name);
                mpt = result[1];
            }
            let impl_deploy = await deploy_contract(hre.artifacts,"LightNode",[],network.name);
            impl = impl_deploy[0];
            let impl_param = LightNode.interface.encodeFunctionData("initialize",[chainId, await getDeployerAddress(network.name), mpt,nodeType]);
            let proxy_deploy = await deploy_contract(hre.artifacts,"LightNodeProxy",[impl_deploy[1],impl_param],network.name);
            node = proxy_deploy[0];
        } else {
            console.log("wallet address is:", wallet.address);
            if(mpt === undefined || mpt === ''){
                let result = await deploy("MPTVerify", {
                                    from: wallet.address,
                                    args: [],
                                    log: true,
                                    contract: "MPTVerify",
                                });
                mpt = result.address;
            }
            let impl_deploy = await deploy("LightNode", {
                            from: wallet.address,
                            args: [],
                            log: true,
                            contract: "LightNode",
                        });
            impl = impl_deploy.address;
            let impl_param = LightNode.interface.encodeFunctionData("initialize",[chainId, wallet.address, mpt,nodeType]);
            let param = hre.ethers.utils.defaultAbiCoder.encode(["address","bytes"],[impl,impl_param])
            let LightNodeProxy = await hre.ethers.getContractFactory("LightNodeProxy");
            let result = await create(salt, LightNodeProxy.bytecode, param,hre.ethers);
            node = result[0];
            const verifyArgs = [chainId, wallet.address, mpt,nodeType]
                .map((arg) => (typeof arg == "string" ? `'${arg}'` : arg))
                .join(" ");
            console.log(`To verify, run: npx hardhat verify --network ${network.name} ${impl} ${verifyArgs}`);
            // await verify(
            //     node,
            //     [chainId, wallet.address, mpt,nodeType],
            //     "contracts/LightNode.sol:LightNode",
            //     hre.run
            // );
        }
        console.log("node  address :", node);
        let d = await readFromFile(network.name);
        d.lightNodeInfos[network.name].proxy = node;
        d.lightNodeInfos[network.name].impl = impl;
        await writeToFile(d);
});

task("node:upgrade", "deploy oracle light node")
    .addParam("node","light node address")
    .addParam("impl", "impl address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const { deployments,network} = hre;
        const { deploy } = deployments;
        let impl = taskArgs.impl;
        let node = taskArgs.node;
        let d = await readFromFile(network.name);
        if(node === 'node'){
            if(d.lightNodeInfos[network.name].proxy === undefined || d.lightNodeInfos[network.name].proxy === ''){
                throw("oracle light node not deploy")
            }
            node = d.lightNodeInfos[network.name].proxy;
        }
        if(network.name === 'Tron' || network.name === 'TronTest'){
            let lightNode = await getTronContractAt(hre.artifacts,"LightNode",node,network.name);
            console.log("old impl :",await lightNode.getImplementation().call());
            let result;
            if(impl === 'impl') {
                let impl_deploy = await deploy_contract(hre.artifacts,"LightNode",[],network.name);
                impl = impl_deploy[0];
                result = await lightNode.upgradeTo(impl_deploy[1]).send()
            } else {
                let hexImpl = impl;
                if(!impl.startsWith("0x")){
                     hexImpl = await toETHAddress(impl,network.name);
                }
                result = await lightNode.upgradeTo(hexImpl).send()
            }
            console.log(result);
            console.log("new impl :",await lightNode.getImplementation().call())
        } else {
            if(impl === 'impl') {
                let l = await deploy("LightNode", {
                    from: wallet.address,
                    args: [],
                    log: true,
                    contract: "LightNode",
                });
               impl = l.address;
            }
            const LightNode = await hre.ethers.getContractFactory("LightNode");
            let proxy = LightNode.attach(node);
            console.log("old impl :",await proxy.getImplementation())
            await (await proxy.upgradeTo(impl)).wait()
            console.log("new impl :",await proxy.getImplementation())
        }
        d.lightNodeInfos[network.name].impl = impl;
        writeToFile(d);
    });

task("node:setMptVerify", "set mpt verify address")
    .addParam("node","light node address")
    .addParam("mpt", "mpt address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        console.log("wallet address is:", wallet.address);
        const {network} = hre;
        let node = taskArgs.node;
        if(node === 'node'){
            let d = await readFromFile(network.name);
            if(d.lightNodeInfos[network.name].proxy === undefined || d.lightNodeInfos[network.name].proxy === ''){
                throw("oracle light node not deploy")
            }
            node = d.lightNodeInfos[network.name].proxy;
        }

        if(network.name === 'Tron' || network.name === 'TronTest'){ 
            let lightNode = await getTronContractAt(hre.artifacts,"LightNode",node,network.name);
            let old_verify = await lightNode.mptVerify().call();
            console.log("old mptVerify address is :", old_verify);
            let mpt = taskArgs.mpt;
            if(!mpt.startsWith("0x")){
                mpt = await toETHAddress(mpt,network.name);
            }
            let result = await lightNode.setMptVerify(mpt).send();
            console.log(result);
            let new_verify = await lightNode.mptVerify().call();
            console.log("new mptVerify address is :", new_verify);
        } else {
            const LightNode = await hre.ethers.getContractFactory("LightNode");
            let proxy = LightNode.attach(node);
            let old_verify = await proxy.mptVerify();
            console.log("old mptVerify address is :", old_verify);
            await (await proxy.setMptVerify(taskArgs.mpt)).wait();
            let new_verify = await proxy.mptVerify();
            console.log("new mptVerify address is :", new_verify);
        }

    });

task("node:setOracle", "set oracle address")
    .addParam("node","light node address")
    .addParam("oracle", "oracle address")
    .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
        let [wallet] = await hre.ethers.getSigners();
        const {network} = hre;
        let node = taskArgs.node;
        if(node === 'node'){
            let d = await readFromFile(network.name);
            if(d.lightNodeInfos[network.name].proxy === undefined || d.lightNodeInfos[network.name].proxy === ''){
                throw("oracle light node not deploy")
            }
            node = d.lightNodeInfos[network.name].proxy;
        }
        if(network.name === 'Tron' || network.name === 'TronTest'){ 
            let lightNode = await getTronContractAt(hre.artifacts,"LightNode",node,network.name);
            let oracle = taskArgs.oracle;
            if(!oracle.startsWith("0x")){
                oracle = await toETHAddress(oracle,network.name);
            }
            let old_oracle = await lightNode.oracle().call();
            console.log("old oracle address is :", old_oracle);
            let result = await lightNode.setOracle(oracle).send();
            console.log(result);
            let new_oracle = await lightNode.oracle().call();
            console.log("new oracle address is :", new_oracle);
        } else {
            console.log("wallet address is:", wallet.address);
            const LightNode = await hre.ethers.getContractFactory("LightNode");
            let proxy = LightNode.attach(node);
            let old_oracle = await proxy.oracle();
            console.log("old oracle address is :", old_oracle);
            await (await proxy.setOracle(taskArgs.oracle)).wait();
            let new_oracle = await proxy.oracle();
            console.log("new oracle address is :", new_oracle);
        }
    });