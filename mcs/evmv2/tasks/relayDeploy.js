
module.exports = async (taskArgs,hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('MAPOmnichainServiceRelayV2', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPOmnichainServiceRelayV2'
    })

    let mosRelay = await ethers.getContract('MAPOmnichainServiceRelayV2');

    console.log("MAPOmnichainServiceRelayV2 address:",mosRelay.address);

    let data = mosRelay.interface.encodeFunctionData("initialize", [taskArgs.wrapped, taskArgs.lightnode]);

    await deploy('MAPOmnichainServiceProxyV2', {
        from: deployer.address,
        args: [mosRelay.address,data],
        log: true,
        contract: 'MAPOmnichainServiceProxyV2',
    })

    let mosRelayProxy = await ethers.getContract('MAPOmnichainServiceProxyV2');

    console.log("MAPCrossChainServiceRelayProxy address:",mosRelayProxy.address);

}