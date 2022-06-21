const BigNumber = require('bignumber.js')
BigNumber.config({ROUNDING_MODE: BigNumber.ROUND_FLOOR})
module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log(
        "Deploying contracts with the account:",
        await deployer.getAddress()
    );

    console.log("Account balance:", (await deployer.getBalance()).toString());


    await deploy('LightNode', {
        from: deployer.address,
        args: [],
        log: true,
        contract: 'LightNode',
    })

    let lightNode = await ethers.getContract('LightNode');

    console.log(lightNode.address)


    await hre.run("verify:verify", {
        address: lightNode.address,
        constructorArguments:[]
    });


    let g1Hex = [
        "0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f2492b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3",
        "0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf581ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b",
        "0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c8121e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869",
        "0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"
    ]

    let blsCode = await hre.ethers.getContractFactory("BlsCode");
    bc = await blsCode.deploy();
    await bc.deployed();

    const g0 = await bc.decodeG1(g1Hex[0]);
    const g1 = await bc.decodeG1(g1Hex[1]);
    const g2 = await bc.decodeG1(g1Hex[2]);
    const g3 = await bc.decodeG1(g1Hex[3]);
    g1List = [
        g0,
        g1,
        g2,
        g3,
    ]

    let addresss = [
        "0xb4e1BC0856f70A55764FD6B3f8dD27F2162108E9",
        "0x7A3a26123DBD9CFeFc1725fe7779580B987251Cb",
        "0x7607c9cdd733d8cDA0A644839Ec2bAc5Fa180eD4",
        "0x65b3FEe569Bf82FF148bddEd9c3793FB685f9333"
    ]

    let _weights = [1, 1, 1, 1]

    let _threshold = 3;

    let _epoch = 1;

    let _epochSize = 1000;

    // function initialize(uint _threshold, address[]  memory _validatorAddresss, G1[] memory _pairKeys,
    //     uint[] memory _weights, uint _epoch, uint _epochSize)

    console.log(_threshold,addresss,g1List,_weights,_epoch,_epochSize)

    // await lightNode.initialize(_threshold, addresss, g1List, _weights, _epoch, _epochSize);


    console.log(await lightNode.getValiditors())

    console.log(await lightNode.getWM())
}

module.exports.tags = ['LightNode']
