module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
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


    let proxy = await deployments.get("MAPOmnichainServiceProxyV2");
    let mosProxy = await ethers.getContractAt('MAPOmnichainServiceV2', proxy.address);

    await (await mosProxy.upgradeTo(mos.address)).wait();

    console.log("MAPOmnichainServiceV2 up success")
}

module.exports.tags = ['MAPOmnichainServiceV2Up']