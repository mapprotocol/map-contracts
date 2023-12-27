import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";



task("setMptVerify","set mpt verify address")
  .addParam("mpt","mpt address")
  .setAction(async (taskArgs, hre: HardhatRuntimeEnvironment) => {
      let [wallet] = await hre.ethers.getSigners();
      console.log("wallet address is:",wallet.address);
      let LightNodeProxy = await hre.deployments.get("LightNodeProxy");
      if(!LightNodeProxy){
        throw("light node not deploy.......");
      }
      console.log("light node proxy address is :", LightNodeProxy.address);
      const LightNode = await hre.ethers.getContractFactory("LightNode");

      let proxy = LightNode.attach(LightNodeProxy.address);

      let old_verify = await proxy.mptVerify();

      console.log("old mptVerify address is :", old_verify);
  
      await (await proxy.setMptVerify(taskArgs.mpt)).wait();
  
      let new_verify = await proxy.mptVerify();
  
      console.log("new mptVerify address is :", new_verify);
  })