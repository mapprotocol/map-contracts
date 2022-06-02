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


    // await hre.run("verify:verify", {
    //     address: lightNode.address,
    //     constructorArguments:[]
    // });


    let g1Hex = [
        "0x14d44a97d2fc3ea62b6dcf2bd857079bd261993152f11aef5dd001db68b20d2d1ba45f117b6530a7aec45d7d90fd4e15d2a62f62b706eaa115aa801caeee294b",
        "0x15b7bcf0accf839170a5d4621282edcf14f4a438f8e53abcead5f0528cb91cb1135fd4e82ede1493ab1209af122e1dc186c885cc96d2413cbc09a58163b91eb9",
        "0x2fd433e93187f6b3d15664ec48073bd73d57c801c4a8bfc1e0e3abd3deefc45619d45ac7ad54df7dda5b8afd6f882c9d9f879dbc6d587f1da5da1751baac729f",
        "0x1b037f39d9f8e74b608a898249cc3d156ff1f0051026388366b85a84aac43bb4068275cd909e16b29f1b3bc97e91ec0a8b95a11b8a574cbc2c9ea142d26c8a49",
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
        "0x1c0eDab88dbb72B119039c4d14b1663525b3aC15",
        "0x16FdBcAC4D4Cc24DCa47B9b80f58155a551ca2aF",
        "0x2dC45799000ab08E60b7441c36fCC74060Ccbe11",
        "0x6C5938B49bACDe73a8Db7C3A7DA208846898BFf5"
    ]

    let _weights = [1, 1, 1, 1]

    let _threshold = 4;

    let _epoch = 0;

    let _epochSize = 4;

    // function initialize(uint _threshold, address[]  memory _validatorAddresss, G1[] memory _pairKeys,
    //     uint[] memory _weights, uint _epoch, uint _epochSize)

    console.log(_threshold,addresss,g1List,_weights,_epoch,_epochSize)

    await lightNode.initialize(_threshold, addresss, g1List, _weights, _epoch, _epochSize);

}

module.exports.tags = ['LightNode']
