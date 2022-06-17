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


        let blsCode = await hre.ethers.getContractFactory("BlsCode");
        bc = await blsCode.deploy();
        await bc.deployed();


        let g1Hex = [
            "0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f2492b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3",
            "0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf581ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b",
            "0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c8121e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869",
            "0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"
        ]


        const g0 = await bc.decodeG1(g1Hex[0]);
        const g1 = await bc.decodeG1(g1Hex[1]);
        const g2 = await bc.decodeG1(g1Hex[2]);
        const g3 = await bc.decodeG1(g1Hex[3]);
        // console.log(g1)
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
        //console.log(wmsc.address);
        await wms.setStateInternal(threshold, g1List, weights, 0)

    });


    it("should verify maximum quorum", async () => {
        assert(await wms.callStatic.isQuorum('0x0f', weights, threshold)); // 1111
    });

    it("should pass 3 of 4", async () => {
        assert(await wms.callStatic.isQuorum('0x07', weights, threshold)); // 0111
        assert(await wms.callStatic.isQuorum('0x0b', weights, threshold)); // 1011
        assert(await wms.callStatic.isQuorum('0x0d', weights, threshold)); // 1101
        assert(await wms.callStatic.isQuorum('0x0e', weights, threshold)); // 1110
    });

    it("should fail 2 of 4", async () => {
        assert.equal(await wms.callStatic.isQuorum('0x03', weights, threshold), false); // 0011
        assert.equal(await wms.callStatic.isQuorum('0x09', weights, threshold), false); // 1001
        assert.equal(await wms.callStatic.isQuorum('0x0a', weights, threshold), false); // 1010
        assert.equal(await wms.callStatic.isQuorum('0x0c', weights, threshold), false); // 1100
    });


    ///function checkSig(
    //         bytes memory bits, bytes memory message, G1 memory sig, G2 memory aggPk, uint256 epoch
    //     ) external returns (bool) {


    it("should check agg sig correctly", async () => {
        const bits = '0x0d'; // 00000111
        const message = "0x83c6030e8eb07e62c2f66a2afebcdcf0a0687ee96f8bfea7482a6ccc502e252502";
        const sign = await bc.decodeG1("0x04017fa35d23482fb010423283569b741e611cfb967ee12673805bdecff8919a1c590bd715dbc1f6a8e133983225b571b8db7c349a2405cb15a266bcf1ed09b2");

        const aggpk = ["0x1e765f27b1bc2822f1543fec2a14530db0eb56175e2cd5bc7c6567ef7d605204",
            "0x10ebefa20bce22d0bce4bba43a151c8eedfed5c6487e98e21907da5384aba903",
            "0x19933df8221f1532677d57c1bafb9220fa098a4774a482b37c98746d32d25ed3",
            "0x1e9dcff0a16587d79dd67fc7917cdce9c2813b4e47adc7a1803e121e7cff9995"]
        const epoch = 0;
        assert(await wms.callStatic.checkSig(bits, message, sign, aggpk, epoch));
    });



    it("should check updateBlockHeader none", async () => {
        const bits = '0x00'; // 00000111
        const _pairKeysAdd = [];
        const _weights = [];
        const epoch = 1;
        //upateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint256 epoch, bytes memory bits)
        assert(await wms.upateValidators(_pairKeysAdd,_weights,epoch,bits));
    });

    it("should check updateBlockHeader add", async () => {
        const bits = '0x10'; // 00000111
        let g4hex ="0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac312";
        const g4 = await bc.decodeG1(g4hex);
        const _pairKeysAdd = [g4];
        const _weights = [1];
        const epoch = 2;
        //upateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint256 epoch, bytes memory bits)
        assert(await wms.upateValidators(_pairKeysAdd,_weights,epoch,bits));
    });


    it("should check updateBlockHeader remove", async () => {
        const bits = '0x00'; // 0x10000
        const _pairKeysAdd = []
        const _weights = [];
        const epoch = 3;
        //upateValidators(G1[] memory _pairKeysAdd, uint[] memory _weights, uint256 epoch, bytes memory bits)
        assert(await wms.upateValidators(_pairKeysAdd,_weights,epoch,bits));
    });
});
