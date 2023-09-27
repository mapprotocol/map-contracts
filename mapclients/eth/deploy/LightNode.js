const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})

const  initializeData = require('./config');

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());
    // console.log(initializeData.initData)
    await deploy('VerifyTool', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'VerifyTool',
    })

    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })


    let VerifyTool = await deployments.get('VerifyTool');
    let LightNode = await deployments.get('LightNode');

    let lightNode = await ethers.getContractAt("LightNode",LightNode.address)
    console.log(lightNode.address)
    //let validatorNum = initializeData.initData.validators;
    let validatorNum = initializeData.validators;
    let g1List = [];
    let addresss = [];
    let weights = []
    for (let i = 0; i < validatorNum.length; i++){
        let temp = [validatorNum[i].g1_pub_key.x,validatorNum[i].g1_pub_key.y];
        g1List.push(temp);
        addresss.push(validatorNum[i].address);
        weights.push((validatorNum[i].weight));
    }

    let threshold = initializeData.threshold;

    let epoch = initializeData.epoch;

    let epochSize = initializeData.epoch_size;


    let data = lightNode.interface.encodeFunctionData("initialize", [threshold, addresss, g1List, weights, epoch, epochSize,VerifyTool.address]);
    console.log("initialize success")


    await deploy('LightNodeProxy', {
        from: deployer.address,
        args: [lightNode.address,data],
        log: true,
        contract: 'LightNodeProxy'
    })

    let lightProxyClient = await deployments.get('LightNodeProxy');

    console.log("lightProxyClient Address",lightProxyClient.address)


}

module.exports.tags = ['LightNode']