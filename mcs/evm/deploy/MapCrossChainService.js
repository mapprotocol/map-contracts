const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

//let lightNodeAddress = "0xF71F0007dDb539e2A506D770bB3a3eE83bD939B9";
let lightNodeAddress = "0x1eD5058d28fCD3ae7b9cfFD0B0B3282d939c4034";

let weth = "0xB59B98DF47432371A36A8F83fC7fd8371ec1300B";
let usdt = "0xdBf63a81d44DA9645498E371A856F9754F4f2c2B";
let mapToken = "0xb245609e5b2a0E52191Cba6314b47C73a0f9f023";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer,wcoin,mapcoin} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('MapCrossChainService', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MapCrossChainService',
    })


    let mcss = await ethers.getContract('MapCrossChainService');


    console.log("MapCrossChainService address:",mcss.address);

    let data = await mcss.initialize(weth,mapToken,lightNodeAddress)
    console.log("MapCrossChainService initialize success");

    await deploy('MapCrossChainServiceProxy', {
        from: deployer.address,
        args: [mcss.address,data.data],
        log: true,
        contract: 'MapCrossChainServiceProxy',
    })

    let mcssP = await ethers.getContract('MapCrossChainServiceProxy');

    console.log("MapCrossChainServiceProxy address:",mcssP.address)



}

module.exports.tags = ['MapCrossChainService']