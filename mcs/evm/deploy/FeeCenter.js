
let mapUsdt = "0xa2FD5Ad95c1F2fC374dF775ad0889eab6d587015";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('FeeCenter', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'FeeCenter',
    })


    let feeCenter = await ethers.getContract('FeeCenter');

    console.log("feeCenter address:",feeCenter.address);

    //feeCenter set setChainTokenGasFee
    await (await feeCenter.connect(deployer).setChainTokenGasFee(34434,mapUsdt,"10000000000000000","10000000000000000000",200)).wait();
    console.log("MapUsdt set fee success");

    await (await feeCenter.setDistributeRate(1,deployer.address,2000)).wait();

}

module.exports.tags = ['FeeCenter']