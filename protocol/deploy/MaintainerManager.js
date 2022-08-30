const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());


    await deploy('MaintainerManager', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MaintainerManager',
    })

    let MaintainerManager = await ethers.getContract('MaintainerManager');

    console.log("MaintainerManager",MaintainerManager.address)

    // await hre.run("verify:verify", {
    //     address: MaintainerManager.address,
    //     constructorArguments:[]
    // });

    await deploy('MaintainerManagerProxy', {
        from: deployer.address,
        args: [MaintainerManager.address,"0x"],
        log: true,
        contract: 'MaintainerManagerProxy',
    })

    let MaintainerManagerProxy = await ethers.getContract('MaintainerManagerProxy');

    console.log("MaintainerManagerProxy",MaintainerManagerProxy.address)


    MaintainerManagerProxy = await ethers.getContractAt("MaintainerManager",MaintainerManagerProxy.address)

    // await MaintainerManagerProxy.initialize();
    console.log(await MaintainerManagerProxy.getAdmin())
    // await MaintainerManagerProxy.changeAdmin(await deployer.getAddress());
    //
    // await MaintainerManagerProxy.addWhiteList("0x0000000000000000000000000000000000000008")
    //
    // console.log(await MaintainerManagerProxy.whiteList("0x0000000000000000000000000000000000000008"));
}

module.exports.tags = ['MaintainerManager']
