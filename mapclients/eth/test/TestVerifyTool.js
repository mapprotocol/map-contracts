const hre = require('hardhat');
const {assert} = require('chai');
const {ethers} = require('hardhat');
const bls254 = require('./blsbn254');
const {BigNumber} = require("ethers");
const proofs = require('./data');
const { rlp } = require('ethereumjs-util');

function buffer2hex(buffer) {
    return '0x' + buffer.toString('hex');
}

function index2key(index, proofLength) {
    const actualkey = [];
    const encoded = buffer2hex(rlp.encode(index)).slice(2);
    console.log(encoded)
    let key = [...new Array(encoded.length / 2).keys()].map(i => parseInt(encoded[i * 2] + encoded[i * 2 + 1], 16));
    console.log(key)
    console.log(actualkey)
    key.forEach(val => {
        if (actualkey.length + 1 === proofLength) {
            actualkey.push(val);
        } else {
            actualkey.push(Math.floor(val / 16));
            actualkey.push(val % 16);
        }
    });
    console.log(actualkey)
    console.log(key)
    return '0x' + actualkey.map(v => v.toString(16).padStart(2, '0')).join('');
}

describe('VerifyTools', function () {
        let VerifyToolClient;
        let verifyToolClient;
        let verifyToolContract;
        let verifyToolContractAddress;



    before(async () => {
        // VerifyToolClient = await hre.ethers.getContractFactory("VerifyTool");
        VerifyToolClient = await hre.ethers.getContractFactory("MPTTest");
        verifyToolClient = await VerifyToolClient.deploy();
        verifyToolContract = await verifyToolClient.deployed()
        verifyToolContractAddress = verifyToolContract.address;
    });

    it("VerifyTools verify ", async () => {
        // assert(await verifyToolContract.getVerifyTrieProof2(proofs.proofOk));

        assert(await verifyToolContract.getVerifyTrieProof2(proofs.proofError3));

        // console.log(index2key(0,2));
        // console.log(index2key(0,2));
        // console.log(index2key(0,3));
        // console.log(index2key(0,1000));
        // console.log(index2key(1000,5));
    });
});
