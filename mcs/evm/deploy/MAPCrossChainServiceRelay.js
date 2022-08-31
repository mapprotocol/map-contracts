const BigNumber = require('bignumber.js')
const {address} = require("hardhat/internal/core/config/config-validation");
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

let managerAddress = "0x9E514B5A4e48Bf8D67C95Ddd586228294b7A58A6";
let ethUsdt = "0xcfC80bEdDb70F12af6dA768FC30e396889DfCe26";
let nearUsdt = "0x6d63735f746f6b656e5f312e6d63732e6d61703030312e746573746e6574";
let mapUsdt = "0xe6EEE0EF1bdE38aC4e577647868FED43a3d0e3A9"

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer,wcoin,mapcoin} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('FeeCenter', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'FeeCenter',
    })

    await deploy('MAPCrossChainServiceRelay', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPCrossChainServiceRelay',
    })

    await deploy('TokenRegister', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'TokenRegister',
    })

    let mcssRelay = await ethers.getContract('MAPCrossChainServiceRelay');
    let feeCenter = await ethers.getContract('FeeCenter');
    let tokenRegister = await ethers.getContract('TokenRegister');

    console.log("MAPCrossChainServiceRelay address:",mcssRelay.address);
    console.log("feeCenter address:",feeCenter.address);
    console.log("tokenRegister address:",tokenRegister.address);

    console.log(wcoin.address);
    console.log(mapcoin.address);

    await (await mcssRelay.initialize(wcoin.address,mapcoin.address,managerAddress)).wait();

    await (await mcssRelay.setFeeCenter(feeCenter.address)).wait();

    await (await mcssRelay.setTokenRegister(tokenRegister.address)).wait();

    await (await mcssRelay.setIdTable(1313161555,1)).wait();


    //tokenRegister set regToken
    await (await tokenRegister.regToken(34434,ethUsdt,mapUsdt)).wait();

    await (await tokenRegister.regToken(1313161555,nearUsdt,mapUsdt)).wait();

    await (await tokenRegister.regToken(212,mapUsdt,ethUsdt)).wait();

    //feeCenter set setChainTokenGasFee
    await (await feeCenter.setChainTokenGasFee(34434,mapUsdt,"10000000000000000","10000000000000000000",200)).wait();

    //MAPCrossChainServiceRelay set setTokenOtherChainDecimals
    await (await mcssRelay.setTokenOtherChainDecimals(mapUsdt,34434,18));
    await (await mcssRelay.setTokenOtherChainDecimals(mapUsdt,212,18));
    await (await mcssRelay.setTokenOtherChainDecimals(mapUsdt,1313161555,24));

    //MAPCrossChainServiceRelay set setVaultBalance
    await (await mcssRelay.setVaultBalance(34434,mapUsdt,"10000000000000000000000000000")).wait();
    await (await mcssRelay.setVaultBalance(1313161555,mapUsdt,"10000000000000000000000000000")).wait();



}

module.exports.tags = ['MAPCrossChainServiceRelay']