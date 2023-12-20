const Caver = require('caver-js')


module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);


    const mainRpcUrl = "https://public-node-api.klaytnapi.com/v1/cypress";
    const testRpcUrl = "https://public-node-api.klaytnapi.com/v1/baobab";

    let deployChainId = hre.network.config.chainId

   // let rpc = mainRpcUrl;
    let rpc = testRpcUrl;
    if (deployChainId === 22776) {
        console.log("deploy id :", deployChainId );
        rpc = mainRpcUrl;
    }

    let caver = new Caver(rpc);

    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let LightNode = await hre.deployments.get('LightNode');

    let lightNode = await ethers.getContractAt('LightNode',LightNode.address);
    console.log('light node implementation address:', lightNode.address);

    let verifyTool = await ethers.getContractAt("VerifyTool",taskArgs.tool)

    let height = Math.trunc(taskArgs.height/3600)*3600;

    console.log("init height:",height);

    let block = await caver.rpc.klay.getBlockByNumber(height);

    let result = await verifyTool.decodeHeaderExtraData(block.extraData);

    let data = lightNode.interface.encodeFunctionData(
        "initialize",
        [result.extData.validators, block.number, taskArgs.tool, taskArgs.mpt]
    );

    console.log("validators",result.extData.validators)

    let lightProxy = await ethers.getContractFactory('LightNodeProxy');

    let initData = await ethers.utils.defaultAbiCoder.encode(
        ["address","bytes"],
        [lightNode.address, data]
    )

    let deployData = lightProxy.bytecode + initData.substring(2);

    console.log("light node salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let factory = await ethers.getContractAt("IDeployFactory", taskArgs.factory)

    console.log("deploy factory address:", factory.address)

    await (await factory.connect(deployer).deploy(hash,deployData,0)).wait();

    let lightProxyAddress = await factory.connect(deployer).getAddress(hash)

    console.log("deployed light node proxy address:", lightProxyAddress)

    let proxy = await ethers.getContractAt('LightNode', lightProxyAddress);

    let owner = await proxy.connect(deployer).getAdmin();

    console.log(`LightNode Proxy contract address is ${lightProxyAddress}, init admin address is ${owner}, deploy contract salt is ${hash}`)
}