const hre = require('hardhat');
const {assert} = require('chai');
const {ethers} = require('hardhat');
const bls254 = require('./blsbn254');
const {BigNumber} = require("ethers");

const formatG1 = (p) => p.x.toHexString() + ',' + p.y.toHexString();
const equalG1 = (p, q) => p.x.eq(q.x) && p.y.eq(q.y);

function convertG1(mclG1) {
    const hex = bls254.g1ToHex(mclG1);
    return {x: BigNumber.from(hex[0]), y: BigNumber.from(hex[1])};
}

function convertG2(mclG2) {
    const hex = bls254.g2ToHex(mclG2);
    return {
        xr: BigNumber.from(hex[0]),
        xi: BigNumber.from(hex[1]),
        yr: BigNumber.from(hex[2]),
        yi: BigNumber.from(hex[3]),
    };
}

describe('WeightedMultiSig', function () {
    let wms;
    let bc;
    let g1List

    const num = 4;
    let res;

    const weights = [1, 1, 1, 1];
    const threshold = 3;

    let signers;
    const message = '0x6162636566676869';

    before(async () => {
        await bls254.init();

        signers = weights.map((w, i) => {
            const key = bls254.newKeyPair(); // pubkey \in G2, secret
            const pkG1 = bls254.g1Mul(key.secret, bls254.g1());

            return {
                index: i,
                weight: w,
                sk: key.secret,
                pkG1: pkG1,
                pkG2: key.pubkey,
            };
        });

        // console.log(signers)


        let blsCode = await  hre.ethers.getContractFactory("BlsCode");
        bc = await blsCode.deploy();
        await  bc.deployed();


        let g1Hex =[
            "0x14d44a97d2fc3ea62b6dcf2bd857079bd261993152f11aef5dd001db68b20d2d1ba45f117b6530a7aec45d7d90fd4e15d2a62f62b706eaa115aa801caeee294b",
            "0x15b7bcf0accf839170a5d4621282edcf14f4a438f8e53abcead5f0528cb91cb1135fd4e82ede1493ab1209af122e1dc186c885cc96d2413cbc09a58163b91eb9",
            "0x2fd433e93187f6b3d15664ec48073bd73d57c801c4a8bfc1e0e3abd3deefc45619d45ac7ad54df7dda5b8afd6f882c9d9f879dbc6d587f1da5da1751baac729f",
            "0x1b037f39d9f8e74b608a898249cc3d156ff1f0051026388366b85a84aac43bb4068275cd909e16b29f1b3bc97e91ec0a8b95a11b8a574cbc2c9ea142d26c8a49",
            ]

        const g0 =  await bc.decodeG1(g1Hex[0]);
        const g1 =  await bc.decodeG1(g1Hex[1]);
        const g2 =  await bc.decodeG1(g1Hex[2]);
        const g3 =  await bc.decodeG1(g1Hex[3]);
        console.log(g1)
        g1List = [
            g0,
            g1,
            g2,
            g3,
        ]


        // signers.forEach(s => console.log(s.index, s.weight, formatG1(convertG1(s.pkG1))));

        const WeightedMultiSig = await hre.ethers.getContractFactory('WeightedMultiSig');
        // wms = await WeightedMultiSig.deploy(threshold, signers.map(s => convertG1(s.pkG1)), weights);
        wms = await WeightedMultiSig.deploy();
        let wmsc = await wms.deployed();
        //function setStateInternal(uint256 _threshold, G1[] memory _pairKeys, uint[] memory _weights, uint256 epoch) public
        console.log(wmsc.address);
        await wms.setStateInternal(threshold, g1List, weights,0)

    });


    it("should verify maximum quorum", async () => {
        assert(await wms.callStatic.isQuorum('0x0f',weights,threshold)); // 1111
    });

    it("should pass 3 of 4", async () => {
        assert(await wms.callStatic.isQuorum('0x07',weights,threshold)); // 0111
        assert(await wms.callStatic.isQuorum('0x0b',weights,threshold)); // 1011
        assert(await wms.callStatic.isQuorum('0x0d',weights,threshold)); // 1101
        assert(await wms.callStatic.isQuorum('0x0e',weights,threshold)); // 1110
    });

    it("should fail 2 of 4", async () => {
        assert.equal(await wms.callStatic.isQuorum('0x03',weights,threshold), false); // 0011
        assert.equal(await wms.callStatic.isQuorum('0x09',weights,threshold), false); // 1001
        assert.equal(await wms.callStatic.isQuorum('0x0a',weights,threshold), false); // 1010
        assert.equal(await wms.callStatic.isQuorum('0x0c',weights,threshold), false); // 1100
    });


    it("should check agg sig correctly", async () => {
        const bits = '0x07'; // 00000111
        const aggPkG2 = bls254.aggreagate(bls254.aggreagate(signers[0].pkG2, signers[1].pkG2), signers[2].pkG2);

        const sigs = signers.map(s => bls254.sign(message, s.sk));
        const aggSig = bls254.aggreagate(bls254.aggreagate(sigs[0].signature, sigs[1].signature), sigs[2].signature);

        assert(await wms.callStatic.checkSig(bits, message, convertG1(aggSig), convertG2(aggPkG2)));
    });
});
