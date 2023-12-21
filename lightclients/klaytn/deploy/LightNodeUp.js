const { LIGHTNODE_SALT,DEPLOY_FACTORY} = process.env;

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());


    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let lightNode = await deployments.get('LightNode');

    console.log(lightNode.address)

    console.log("LightNode salt:", LIGHTNODE_SALT);

    let factory = await ethers.getContractAt("IDeployFactory",DEPLOY_FACTORY)

    console.log("deploy factory address:",factory.address)

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(LIGHTNODE_SALT));

    let lightAddress = await factory.getAddress(hash);

    let lightProxy = await ethers.getContractAt('LightNode',lightAddress);

    console.log("LightNodeProxy proxy address:", lightAddress);

    await (await lightProxy.upgradeTo(lightNode.address)).wait();

    console.log("LightNode up success");
}

module.exports.tags = ['LightNodeUp']
