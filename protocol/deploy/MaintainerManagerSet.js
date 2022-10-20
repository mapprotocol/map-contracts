const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());


    let MaintainerManager = await ethers.getContract('MaintainerManager');

    console.log("MaintainerManager",MaintainerManager.address)

    let MaintainerManagerProxy = await ethers.getContract('MaintainerManagerProxy');
    console.log("MaintainerManagerProxy",MaintainerManagerProxy.address)

    MaintainerManager = await ethers.getContractAt("MaintainerManager",MaintainerManagerProxy.address)

    let maintainer = "";
    let add = true;


    if (add) {
       await MaintainerManager.addWhiteList(maintainer);
    }else{
        await MaintainerManager.removeWhiteList(maintainer);
    }

}

module.exports.tags = ['MaintainerManagerSet']
