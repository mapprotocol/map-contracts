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


     let verifyTool = await ethers.getContract('VerifyTool');
    let lightNode = await ethers.getContract('LightNode');

    //ROPSTEN
    //0x04b12c39a37230c99b9f0D57902509179C4BCd60
    // console.log(verifyTool.address)
    //0x277a38555d889cF76E9bc8f90Ee312eB7605Ce4D
    console.log(lightNode.address)


    // await hre.run("verify:verify", {
    //     address: lightNode.address,
    //     constructorArguments:[]
    // });

    // let g1Hex = [
    //     "0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f2492b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3",
    //     "0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf581ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b",
    //     "0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c8121e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869",
    //     "0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"
    // ]
    let g1Hex =
        [
            "0x01370ecd3f4871a718079cb799ed57597b6087eb09811fae7635f541a0b14c571b327c6f9d07f6f2b666e341fa7cb3531ee510da50fedc567739a7040a1dc696",
            "0x2dc393cb4e1d6bb5e26c4fef0ccdde874535af1da42f64b34525a399dc1bbe621291bd0437dbb1f7ea7737ad515546b8f6b696ea0b9f6f49d5f6c039259ae778",
            "0x2801781ffcf2371c911090b1dfe626a7b4e745810f30d545e45b965674bee6b323ef4f51b21bd4d141e484ff8f9d5becddc4ffe0d432a80d59b982aab1f9e575",
            "0x1d330a79f1374d37c618bcb34edc38f99935a9f44d3885672232495e22fce1512b742d040ff3e9a996b79406cc4f18fc6c9b4a28ee7c3e88590406259f404531"
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

    let addresss =
        [
            "0xec3e016916ba9f10762e33e03e8556409d096fb4",
            "0x6f08db5ba52d896f2472eb49580ac6d8d0351a66",
            "0x2f3079a1c1c0995a1c9803853d1b8444cce0aa9f",
            "0x096bf1097f3af73b716eab545001d97b2cf1fb20"
        ]


    // let addresss = [
    //     "0xb4e1BC0856f70A55764FD6B3f8dD27F2162108E9",
    //     "0x7A3a26123DBD9CFeFc1725fe7779580B987251Cb",
    //     "0x7607c9cdd733d8cDA0A644839Ec2bAc5Fa180eD4",
    //     "0x65b3FEe569Bf82FF148bddEd9c3793FB685f9333"
    // ]

    let _weights = [1, 1, 1, 1]

    let _threshold = 3;

    let _epoch = 1;

    let _epochSize = 1000;

    // function initialize(uint _threshold, address[]  memory _validatorAddresss, G1[] memory _pairKeys,
    //     uint[] memory _weights, uint _epoch, uint _epochSize)

    //console.log(_threshold,addresss,g1List,_weights,_epoch,_epochSize)

    await lightNode.initialize(_threshold, addresss, g1List, _weights, _epoch, _epochSize,verifyTool.address);
    console.log("initialize success")

}

module.exports.tags = ['LightNode']
