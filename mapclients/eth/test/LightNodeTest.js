const { ethers } = require("hardhat");
const proofs = require('./data');
require("solidity-coverage");

describe("LightNode start test", function () {

    let lightClient;
    let lightNodeContract;
    let lightNodeContractAddress;

    let blsCode;
    let bc;

    let g1List;

    it("deploy LightNode",async function () {
        const LightClient = await ethers.getContractFactory("LightNode");
        lightClient = await LightClient.deploy();
        lightNodeContract = await lightClient.deployed()
        lightNodeContractAddress = lightNodeContract.address;
    });

    it('initialize ', async function () {
        let g1Hex = [
            "0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f2492b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3",
            "0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf581ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b",
            "0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c8121e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869",
            "0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311"
        ]

        blsCode = await ethers.getContractFactory("BlsCode");
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

        let _epoch = 123;

        let _epochSize = 1000;

        await lightClient.initialize(_threshold, addresss, g1List, _weights, _epoch, _epochSize);
    });





    it('updateBlockHeader', async function () {

        await lightClient.updateBlockHeader(proofs.header123,proofs.aggpk123);

        let wmAddress = await lightClient.getWM()

        console.log(wmAddress);

        console.log(await lightClient.getValiditors());

        await lightClient.updateBlockHeader(proofs.header124,proofs.aggpk124);

        await lightClient.updateBlockHeader(proofs.header125,proofs.aggpk125);

        await lightClient.updateBlockHeader(proofs.header126,proofs.aggpk126);

        console.log(await lightClient.callStatic.verifyProofData(proofs.provedata2568));

        console.log(await lightClient.callStatic.verifyProofData(proofs.provedata4108));
        console.log(await lightClient.callStatic.verifyProofData(proofs.provedataTestProof));
        console.log(await lightClient.callStatic.verifyProofData(proofs.provedataTestHeader));
        console.log(await lightClient.callStatic.verifyProofData(proofs.provedataTestSig));
        console.log(await lightClient.callStatic.verifyProofData(proofs.provedata55342));

    });


    let LightClientDelete
    let lightClientDelete;
    let lightNodeContractDelete;
    let lightNodeContractDeleteAddress;

    let blsCodeDelete;
    let bcDelete;


    it('delete deploy', async function () {

        LightClientDelete = await ethers.getContractFactory("LightNode");
        lightClientDelete = await LightClientDelete.deploy();
        lightNodeContractDelete = await lightClientDelete.deployed()
        lightNodeContractDeleteAddress = lightNodeContractDelete.address;
        console.log(lightNodeContractDeleteAddress);


        blsCodeDelete = await ethers.getContractFactory("BlsCode");
        bcDelete = await blsCodeDelete.deploy();
        await bcDelete.deployed();

    });

    it('delete committee member ', async function () {
        let g1Hex = [
            "0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f2492b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3",
            "0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf581ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b",
            "0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c8121e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869",
            "0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311",
            "0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116",
            "0x11902b17829937be3f969e58f386ddfd7ef19065da959cba0caeda87a298ce2d2f79adf719a0099297bb8fb503f25b5d5c52fad67ab7a4a03cb74fe450f4decd"
        ]

        const g0 = await bcDelete.decodeG1(g1Hex[0]);
        const g1 = await bcDelete.decodeG1(g1Hex[1]);
        const g2 = await bcDelete.decodeG1(g1Hex[2]);
        const g3 = await bcDelete.decodeG1(g1Hex[3]);
        const g4 = await bcDelete.decodeG1(g1Hex[4]);
        const g5 = await bcDelete.decodeG1(g1Hex[5]);
        let g1ListDelete = [
            g0,
            g1,
            g2,
            g3,
            g4,
            g5
        ]

        let addresss = [
            "0xb4e1BC0856f70A55764FD6B3f8dD27F2162108E9",
            "0x7A3a26123DBD9CFeFc1725fe7779580B987251Cb",
            "0x7607c9cdd733d8cDA0A644839Ec2bAc5Fa180eD4",
            "0x65b3FEe569Bf82FF148bddEd9c3793FB685f9333",
            "0x98EFA292822eB7b3045C491e8ae4E82B3b1AC005",
            "0x4cA1A81e4c46B90eC52371c063d5721dF61E7e12"
        ]
        let _weights = [1, 1, 1, 1, 1, 1]

        let _threshold = 4;

        let _epoch = 189;

        let _epochSize = 1000;

        await lightClientDelete.initialize(_threshold, addresss, g1ListDelete, _weights, _epoch, _epochSize);

        let wmAddressDelete = await lightClientDelete.getWM()

        console.log("newAddress:" + wmAddressDelete);


        await lightClientDelete.updateBlockHeader(proofs.header189,proofs.aggpk189);

        await lightClientDelete.updateBlockHeader(proofs.header190,proofs.aggpk190);

        await lightClientDelete.updateBlockHeader(proofs.header191,proofs.aggpk191);

    });

    it('verifyProofData deploy', async function () {

        LightClientDelete = await ethers.getContractFactory("LightNode");
        lightClientDelete = await LightClientDelete.deploy();
        lightNodeContractDelete = await lightClientDelete.deployed()
        lightNodeContractDeleteAddress = lightNodeContractDelete.address;
        console.log(lightNodeContractDeleteAddress);


        blsCodeDelete = await ethers.getContractFactory("BlsCode");
        bcDelete = await blsCodeDelete.deploy();
        await bcDelete.deployed();

    });

    it('verifyProofData ', async function () {
        let g1Hex = [
            "0x13524ec450b9ac611fb332a25b6c2eb436d13ac8a540f69a50d6ff8d4fe9f2492b7d0f6e80e80e9b5f9c7a9fa2d482c2e8ea6c1657057c5548b7e30412d48bc3",
            "0x0e3450c5b583e57d8fe736d276e9e4bb2ce4b38a5e9ac77b1289ba14a5e9cf581ce786f52d5bd0e77c1eacfa3dd5df0e22464888fa4bfab6eff9f29e8f86084b",
            "0x2f6dd4eda4296d9cf85064adbe2507901fcd4ece425cc996827ba4a2c111c8121e6fe59e1d18c107d480077debf3ea265a52325725a853a710f7ec3af5e32869",
            "0x05fde1416ab5b30e4b140ad4a29a52cd9bc85ca27bd4662ba842a2e22118bea60dc32694f317d886daac5419b39412a33ee89e07d39d557e4e2b0e48696ac311",
            "0x11902b17829937be3f969e58f386ddfd7ef19065da959cba0caeda87a298ce2d2f79adf719a0099297bb8fb503f25b5d5c52fad67ab7a4a03cb74fe450f4decd"
        ]

        const g0 = await bcDelete.decodeG1(g1Hex[0]);
        const g1 = await bcDelete.decodeG1(g1Hex[1]);
        const g2 = await bcDelete.decodeG1(g1Hex[2]);
        const g3 = await bcDelete.decodeG1(g1Hex[3]);
        const g4 = await bcDelete.decodeG1(g1Hex[4]);
        let g1ListDelete = [
            g0,
            g1,
            g2,
            g3,
            g4
        ]

        let addresss = [
            "0xb4e1BC0856f70A55764FD6B3f8dD27F2162108E9",
            "0x7A3a26123DBD9CFeFc1725fe7779580B987251Cb",
            "0x7607c9cdd733d8cDA0A644839Ec2bAc5Fa180eD4",
            "0x65b3FEe569Bf82FF148bddEd9c3793FB685f9333",
            "0x4cA1A81e4c46B90eC52371c063d5721dF61E7e12"
        ]
        let _weights = [1, 1, 1, 1, 1]

        let _threshold = 4;

        let _epoch = 203;

        let _epochSize = 1000;

        await lightClientDelete.initialize(_threshold, addresss, g1ListDelete, _weights, _epoch, _epochSize);



        console.log(await lightClientDelete.callStatic.verifyProofData(proofs.provedata202351));
    });


});
