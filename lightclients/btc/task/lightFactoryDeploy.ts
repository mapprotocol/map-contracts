
module.exports = async (taskArgs:any,hre:any) => {
    const { ethers } = hre;
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let lightNode = await ethers.getContract('LightNode');
    console.log('light node implementation address:', lightNode.address);

    let data = lightNode.interface.encodeFunctionData(
        "initialize",
        [taskArgs.header,taskArgs.height]
    );

    let lightProxy = await ethers.getContractFactory('LightNodeProxy');

    let initData = await ethers.utils.defaultAbiCoder.encode(
        ["address","bytes"],
        [lightNode.address,data]
    )

    let deployData = lightProxy.bytecode + initData.substring(2);

    console.log("light node salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let factory = await ethers.getContractAt("IDeployFactory",taskArgs.factory)

    console.log("deploy factory address:",factory.address)

    await (await factory.connect(deployer).deploy(hash,deployData,0,{gasLimit: 5000000})).wait();

    let lightProxyAddress = await factory.connect(deployer).getAddress(hash)

    console.log("deployed light node proxy address:", lightProxyAddress)

    let proxy = await ethers.getContractAt('LightNode', lightProxyAddress);

    let owner = await proxy.connect(deployer).getAdmin();

    console.log(`BTC LightNode Proxy contract address is ${lightProxyAddress}, init admin address is ${owner}, deploy contract salt is ${hash}`)
}