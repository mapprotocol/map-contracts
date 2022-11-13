module.exports = async (taskArgs, hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    await deploy('MAPOmnichainServiceV2', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPOmnichainServiceV2',
    })

    let mos = await ethers.getContract('MAPOmnichainServiceV2');

    console.log("MAPOmnichainServiceV2 address:", mos.address);


    let data = mos.interface.encodeFunctionData("initialize", [taskArgs.wrapped, taskArgs.lightnode]);

    await deploy('MAPOmnichainServiceProxyV2', {
        from: deployer.address,
        args: [mos.address, data],
        log: true,
        contract: 'MAPOmnichainServiceProxyV2',
    })

    let mosProxy = await ethers.getContract('MAPOmnichainServiceProxyV2');

    console.log("MapCrossChainServiceProxy address:", mosProxy.address)
}