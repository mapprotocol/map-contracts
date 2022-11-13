

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    await deploy('MAPOmnichainServiceRelayV2', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPOmnichainServiceRelayV2',
    })

    let mosRelay = await ethers.getContract('MAPOmnichainServiceRelayV2');

    console.log("MAPOmnichainServiceRelayV2 up address:",mosRelay.address);

    let proxy = await deployments.get("MAPOmnichainServiceProxyV2")

    let mosRelayProxy = await ethers.getContractAt('MAPOmnichainServiceRelayV2',proxy.address);

    await (await mosRelayProxy.upgradeTo(mosRelay.address)).wait();

    console.log("MAPOmnichainServiceRelayV2 up success");

}

module.exports.tags = ['MAPOmnichainServiceRelayV2Up']