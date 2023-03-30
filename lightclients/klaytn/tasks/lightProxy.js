let fs = require("fs");
const Caver = require('caver-js')


module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    const mainRpcUrl = "https://public-node-api.klaytnapi.com/v1/cypress";
    const testRpcUrl = "https://public-node-api.klaytnapi.com/v1/baobab";

    let rpc = mainRpcUrl;
    if (taskArgs.rpc === "test"){
        rpc = testRpcUrl;
    }

    let caver = new Caver(rpc);

    let height = Math.trunc(taskArgs.height/3600)*3600;

    console.log("init height:",height);

    let block = await caver.rpc.klay.getBlockByNumber(height);

    let lightNode = await ethers.getContract('LightNode');

    let result = await lightNode.decodeHeaderExtraData(block.extraData);

    let data = lightNode.interface.encodeFunctionData("initialize",
        [result.extData.validators,block.number]);

    console.log("validators",result.extData.validators)

    await deploy('LightNodeProxy', {
        from: deployer.address,
        args: [lightNode.address,data],
        log: true,
        contract: 'LightNodeProxy',
    })

    let lightProxy = await ethers.getContract('LightNodeProxy');

    console.log("lightProxyClient Address",lightProxy.address)
}