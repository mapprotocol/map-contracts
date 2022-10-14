const BigNumber = require('bignumber.js')
const {address} = require("hardhat/internal/core/config/config-validation");
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

let managerAddress = "0xCDD415445ddBFeC30a7B7EF73E20b1d5fFc3Ae10";
let MDT = "0xfC109d725a41fFA5E50001c0B464438efBC197f2";
let WMAP = "0xC38D963541E07e552258C014CB22e35f26Fe355B";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    await deploy('MAPCrossChainServiceRelay', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'MAPCrossChainServiceRelay',
    })

    let mcssRelay = await ethers.getContract('MAPCrossChainServiceRelay');

    console.log("MAPCrossChainServiceRelay address:",mcssRelay.address);

    let data = await mcssRelay.initialize(WMAP,MDT,managerAddress);
    console.log("init success");

    //0xc0c53b8b000000000000000000000000680984f78fc18cd566c8648f318c5524fc01d469000000000000000000000000ec4ea15caef4817f020a36a32850537065eceb380000000000000000000000003174b169faa275244ca308a6f939cab5502ba841
    await deploy('MAPCrossChainServiceRelayProxy', {
        from: deployer.address,
        args: [mcssRelay.address,data.data],
        log: true,
        contract: 'MAPCrossChainServiceRelayProxy',
    })

    let mcssRelayP = await ethers.getContract('MAPCrossChainServiceRelayProxy');

    console.log("MAPCrossChainServiceRelayProxy address:",mcssRelayP.address);

}

module.exports.tags = ['MAPCrossChainServiceRelay']