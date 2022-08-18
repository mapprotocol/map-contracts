const { ethers } = require("hardhat");
const proofs = require('./data');
const {expect} = require('chai');
require("solidity-coverage");

describe("LightNode start test", function () {

    let owner;
    let addr1;

    let lightClient;
    let lightNodeContract;
    let lightNodeContractAddress;

    let VerifyToolClient;
    let verifyToolClient;
    let verifyToolContract;
    let verifyToolContractAddress;


    let blsCode;
    let bc;

    let g1List;

    let datInit;

    beforeEach(async function () {
       [owner,addr1]  = await ethers.getSigners();
        VerifyToolClient = await ethers.getContractFactory("VerifyTool");
        verifyToolClient = await VerifyToolClient.deploy();
        verifyToolContract = await verifyToolClient.deployed()
        verifyToolContractAddress = verifyToolContract.address;
    });

    // it("deploy VerifyTool",async function () {
    //     VerifyToolClient = await ethers.getContractFactory("VerifyTool");
    //     verifyToolClient = await VerifyToolClient.deploy();
    //     verifyToolContract = await verifyToolClient.deployed()
    //     verifyToolContractAddress = verifyToolContract.address;
    // });



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

       let data =  await lightClient.initialize(_threshold, addresss, g1List, _weights, _epoch, _epochSize,verifyToolContractAddress);
       datInit = data.data;
    });



    it('updateBlockHeader', async function () {

        await lightClient.updateBlockHeader(proofs.header123,proofs.aggpk123);


        console.log(await lightClient.getValiditors());

        await lightClient.updateBlockHeader(proofs.header124,proofs.aggpk124);

        await lightClient.updateBlockHeader(proofs.header125,proofs.aggpk125);

        await lightClient.updateBlockHeader(proofs.header126,proofs.aggpk126);


        console.log(await lightClient.callStatic.verifyProofData(await lightClient.getBytes(proofs.provedata2568)));

        console.log(await lightClient.callStatic.verifyProofData( await lightClient.getBytes(proofs.provedata4108)));
        console.log(await lightClient.callStatic.verifyProofData(await lightClient.getBytes(proofs.provedataTestProof)));
        console.log(await lightClient.callStatic.verifyProofData(await lightClient.getBytes(proofs.provedataTestHeader)));
        console.log(await lightClient.callStatic.verifyProofData(await lightClient.getBytes(proofs.provedataTestSig)));
        console.log(await lightClient.callStatic.verifyProofData(await lightClient.getBytes(proofs.provedata55342)));

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

        await lightClientDelete.initialize(_threshold, addresss, g1ListDelete, _weights, _epoch, _epochSize,verifyToolContractAddress);


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

        await lightClientDelete.initialize(_threshold, addresss, g1ListDelete, _weights, _epoch, _epochSize,verifyToolContractAddress);

        console.log(await lightClientDelete.callStatic.verifyProofData(await lightClient.getBytes(proofs.provedata202351)));
    });

    it('_authorizeUpgrade test', async function () {

        const LightClientP = await ethers.getContractFactory("LightNode");
        let lightClientP = await LightClientP.deploy();
        await lightClientP.deployed()
        console.log("lightClientP:",lightClientP.address);

        let g1ListP =
            [
                [
                    "0x01370ecd3f4871a718079cb799ed57597b6087eb09811fae7635f541a0b14c57",
                    "0x1b327c6f9d07f6f2b666e341fa7cb3531ee510da50fedc567739a7040a1dc696"
                ],
                [
                    "0x2dc393cb4e1d6bb5e26c4fef0ccdde874535af1da42f64b34525a399dc1bbe62",
                    "0x1291bd0437dbb1f7ea7737ad515546b8f6b696ea0b9f6f49d5f6c039259ae778"
                ],
                [
                    "0x2801781ffcf2371c911090b1dfe626a7b4e745810f30d545e45b965674bee6b3",
                    "0x23ef4f51b21bd4d141e484ff8f9d5becddc4ffe0d432a80d59b982aab1f9e575"
                ],
                [
                    "0x1d330a79f1374d37c618bcb34edc38f99935a9f44d3885672232495e22fce151",
                    "0x2b742d040ff3e9a996b79406cc4f18fc6c9b4a28ee7c3e88590406259f404531"

                ]
            ]
        let addresss =
            [
                "0xec3e016916ba9f10762e33e03e8556409d096fb4",
                "0x6f08db5ba52d896f2472eb49580ac6d8d0351a66",
                "0x2f3079a1c1c0995a1c9803853d1b8444cce0aa9f",
                "0x096bf1097f3af73b716eab545001d97b2cf1fb20"
            ]
        let _weights = [1, 1, 1, 1]
        let _threshold = 3;
        let _epoch = 1;
        let _epochSize = 1000;
        console.log(verifyToolContract.address);
      //  let initD = await lightClientP.initialize(_threshold, addresss, g1ListP, _weights, _epoch, _epochSize,verifyToolContract.address);
        console.log("initialize success")
        //console.log(initD)

        let initDateP = "0x58e25aeb000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000322813fd9a801c5507c9de605d63cea4f2ce6c440000000000000000000000000000000000000000000000000000000000000004000000000000000000000000ec3e016916ba9f10762e33e03e8556409d096fb40000000000000000000000006f08db5ba52d896f2472eb49580ac6d8d0351a660000000000000000000000002f3079a1c1c0995a1c9803853d1b8444cce0aa9f000000000000000000000000096bf1097f3af73b716eab545001d97b2cf1fb20000000000000000000000000000000000000000000000000000000000000000401370ecd3f4871a718079cb799ed57597b6087eb09811fae7635f541a0b14c571b327c6f9d07f6f2b666e341fa7cb3531ee510da50fedc567739a7040a1dc6962dc393cb4e1d6bb5e26c4fef0ccdde874535af1da42f64b34525a399dc1bbe621291bd0437dbb1f7ea7737ad515546b8f6b696ea0b9f6f49d5f6c039259ae7782801781ffcf2371c911090b1dfe626a7b4e745810f30d545e45b965674bee6b323ef4f51b21bd4d141e484ff8f9d5becddc4ffe0d432a80d59b982aab1f9e5751d330a79f1374d37c618bcb34edc38f99935a9f44d3885672232495e22fce1512b742d040ff3e9a996b79406cc4f18fc6c9b4a28ee7c3e88590406259f40453100000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001"
        const LightClientP1 = await ethers.getContractFactory("LightNode");
        let lightClientP1 = await LightClientP1.deploy();
        await lightClientP1.deployed()
        console.log("lightClientP1:",lightClientP1.address);

        const LightProxyClient = await ethers.getContractFactory("LightNodeProxy");
        let lightProxyClient = await LightProxyClient.deploy(lightClientP.address,initDateP);
        await lightProxyClient.deployed()

        let proxy = LightClientP.attach(lightProxyClient.address);
        let blockHeader1000 = [
            "0x44a7aae6606e0175464c86293501bf36cf546942ebb97eb030286fb54c59530f",
            "0x2f3079A1C1c0995A1C9803853D1B8444cCe0Aa9F",
            "0xc2d3f132647f84a78663ac577dc87be7e82c4a881b7a7aed1bc914e1a40e4190",
            "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "0x46343cca2221d659d92a75f7e60203335b67020815cbe7f7de6632461f82e61f",
            "0x02000000000000000000040000000000000000000000000000000000000020000000000000800000000000000000000000000000000000000200000000000000000000000004040000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000a00000800000020000008000000000010000000000000000000000000010000001000000000000000000000040000000000000000000000000000000000000000800000000000000000000000000000000002000000000000000000000080000000000000201000000000000400000000000000000000000000000000000000008000000000000000000000",
            "1000",
            "8000000",
            "0",
            "1658400603",
            "0xd7820503846765746888676f312e31352e36856c696e75780000000000000000f8d3c0c0c080b841734ccc6518bb656d19101c911c874a3ab471fa696cb3955cc88ea9229f0e21381fbb670ae9363dccbbe190d81673e05542a2e9399bc4fbf7e1576a7a3727379400f84407b8402315d1ac45bbf303dabd6ffb2f1f76b015a2fdedafd7fd4b9496db504b9934e520778018028ca74849c6f7d22b0f342436db25fbdcd6f12df63c3b7db58b6fd480f8440fb8402bc03a8a2c9385b3eaffdff58dabf07d9cc1f2c9ce255aa8f671f381cda64d8b22cd53b21401e46a95dd100ee00a477ebaacbd9190e0c18a5861f7e5855dac6080",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000",
            "100000000000"
        ]

        let aggPk1000 = [
            "0x2e5ab621bfb5412851a2b259422fbd7f511f50a7ebbc29c956f27f0a9e52d038",
            "0x2f1bae282fb546b447ac050a3dfb5e239e265834ee9314dfacdcfa5ecc828817",
            "0x2fcef1b22831c9f68dfba54e16ead7453e6f9a1c2f26123b1ca9a70464737659",
            "0x0268e7bb2cf277fcd2fad3be0b074518a19a468f45327fda1bba22c139c13717"
        ]

        await proxy.updateBlockHeader(blockHeader1000,aggPk1000);

        await  proxy.upgradeTo(lightClientP1.address);

        expect(await proxy.getImplementation()).to.equal(lightClientP1.address);

        await proxy.changeAdmin(addr1.address);

        expect(await proxy.getAdmin()).to.equal(addr1.address);

        await proxy.connect(addr1).upgradeTo(lightClientP.address);

        console.log(await proxy.epochSize());

    });


});
