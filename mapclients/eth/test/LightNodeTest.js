const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Relayer,LightNode start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;

    let verifyProof;
    let verifyProofContract;

    let lightClient;
    let lightNodeContract;
    let lightNodeContractAddress;


     let header =
        [
            '0x55c7e51c72094013140782a4b58a8ce54cf0e269ef40090bed532915ff8bcba3',
            '0x16FdBcAC4D4Cc24DCa47B9b80f58155a551ca2aF',
            '0x9f1882228048b3b6fcd16e33ded392dda1aaf06548cfa5fd035c6f71cfab5c27',
            '0xd3ef08509b58dd0bc78ad813f8d0dcbb0815bacd178418a7e1349159133015b4',
            '0x3d0a4f1e4cb1ad9502dc205b56c8d147d27e88f66c784e4ea7dcff83bedd1aed',
            '0x00000800000000000000000000000000000000000000000040000000000020000000000000000000200000000000000000000000000000000100000000000000000000000000000000000008400000000000000000000000000000000000000000000000000020000000800000000000000000000000000000000010000000000000000000000000000000000000000000000000000000020000020000810000000000000000000000000000000000000000000000000000000000000000001000000002000000000000000000000000000000000000000000000000000000000008000000000000000400000000000000000000000000000000000000000000',
            '53828',
            '8000000',
            '105029',
            '1653550219',
            '0xd7820304846765746888676f312e31352e36856c696e75780000000000000000f8d3c0c0c080b841ae5d714f8dfc16a910385e654a5e686388d9aeaeeefa21f3517e3cecc0a82071090f3f3fc68c12189fea8b87811e58ca2a25e5b9f682d3fe14192599a3e5f35201f8440bb8400dfd6c83fad64379b4a012781ce61bf565f351da95600e476c98d3f69413799a02f06decf57ccde54cd821ba944192653fb45f5851266fa994e1edbb9e04835d80f8440fb8402140e41b34e47f8619a67c0de3da7db6e8c323baf0bd678b6d65917dbdfbb8de2b69b1a0f4a64e50103b048573f1e132f461113338a872d30266b4654993299c80',
            '0x0000000000000000000000000000000000000000000000000000000000000000',
            '0x0000000000000000',
            '100000000000'
        ]

    let headerHash;

    let logs = [
        [
            "0x1f9D9D9B34D26e087EE00f61896f3E01dD929843",
            [
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                "0x0000000000000000000000006621f2b6da2bed64b5ffbd6c5b2138547f44c8f9",
                "0x0000000000000000000000006d6247501b822fd4eaa76fcb64baea360279497f"
            ] ,
            "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000"
        ]

    ];

    let txReceipt = [
        "2",
        "0x01",
        "37535",
        "0x00000800000000000000000000000000000000000000000040000000000000000000000000000000200000000000000000000000000000000100000000000000000000000000000000000008400000000000000000000000000000000000000000000000000020000000800000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000002000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000",
        logs
    ];

    let g2 = ["0","0","0","0"]

    let provedata =
        [
            header,
            g2,
            txReceipt,
            "0x0800",
            [
                "0xf851a0fc7c25e8a18de063cc65a2d3914f1482b11a96d8058bc3e7594e26482c0a49eb80808080808080a05b6a980042de579f163badeeac9510c3ef0bdb555dfc53d61eb8bbb33faec3a28080808080808080",
                "0xf901ae30b901aa02f901a60182929fb9010000000800000000000000000000000000000000000000000040000000000000000000000000000000200000000000000000000000000000000100000000000000000000000000000000000008400000000000000000000000000000000000000000000000000020000000800000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000002000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000f89df89b941f9d9d9b34d26e087ee00f61896f3e01dd929843f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000006621f2b6da2bed64b5ffbd6c5b2138547f44c8f9a00000000000000000000000006d6247501b822fd4eaa76fcb64baea360279497fa00000000000000000000000000000000000000000000000000de0b6b3a7640000"
            ]


        ];



    let extraData = "0xd7820302846765746888676f312e31352e36856c696e75780000000000000000f8aec0c080b841f8cbb131f80ff58e82ba6ae89bbb5b3e5d5c87971ac5c6f04e9bd736d56a3aeb485574397571244a4b9da0ecd8f69111c67d9ab0ae63b6e817a3878c26472c2a01f30eb0bc888bf32d63fd40c5a3e247b426a6b7270029acc65e5b31cefcd56769215cd7f309b1c6fe850a978cedb7faf1ecd90080f30fb0f0e38417556f6d582f342807da189d474271507ddb967e10b473ee5d9709b6be32a5ded4420593744c658d3b3db77d0080"
    let extra;


    beforeEach(async function () {

        [owner, addr1, addr2, addr3, addr4, addr5] = await ethers.getSigners();


    });

    it('verifyProof', async function () {

        const VerifyProofInfo = await ethers.getContractFactory("VerifyProof");
        verifyProof = await VerifyProofInfo.deploy();
        verifyProofContract = await verifyProof.deployed()

        console.log(verifyProofContract.address);


    });

    it("initialize,_encodeTxReceipt,_queryLog",async function () {
        const LightClient = await ethers.getContractFactory("LightNode");
        lightClient = await LightClient.deploy();
        lightNodeContract = await lightClient.deployed()
        lightNodeContractAddress = lightNodeContract.address;


    });


    it('verifyProofData', async function () {
        // console.log(verifyProofContract.address);
        // console.log(await lightClient.verifyProof());
        await lightClient.setVerifyer(verifyProofContract.address);

        //console.log(await lightClient.verifyProof());


        // let statusInfo = await lightClient.getStatus("true");
        // console.log(statusInfo);


        console.log(await lightClient.getVerifyTrieProof(provedata));



    });

    it('_encodeHeader,_decodeHeader',async function () {

        headerHash = await lightClient._encodeHeader(header);
        //console.log("header:" + headerHash);

        //await  lightClient._decodeHeader(headerPlp);

    });

    it('_decodeExtraData', async function () {
        //let headerInfo = await lightClient._decodeHeader(headerHash);
        //console.log(headerInfo);
        //extra = await lightClient._decodeExtraData(headerInfo.extraData);
        // console.log(extra);

        //let extraDataPre = await lightClient.splitExtra(headerInfo.extraData);
        // console.log(extraDataPre);

        // let extraDeleteAgg = await lightClient._deleteAgg(extra,extraDataPre);
        // console.log("agg-----" + extraDeleteAgg);

        //let extraDeleteAggTest = await lightClient._deleteAggTest(extra,extraDataPre);
        //console.log("agg-----" + extraDeleteAggTest);

        //let extraDeleteAgg1 = await lightClient._deleteAgg(extra,headerInfo.extraData);
        //console.log("agg1-----" + extraDeleteAgg1);

        // let extraDleleteAanAggData1 = await lightClient._deleteSealAndAgg(extra,extraDataPre);
        // console.log( "data1-----"+extraDleleteAanAggData1);


        // let extraDleleteAanAggData2 = await lightClient._deleteSealAndAgg(extra,extraDeleteAgg);
        // console.log("data2-----" + extraDleleteAanAggData2)

        //let istbs =  await lightClient._encodeAggregatedSeal("0","0x","0");
        //istbs = "0xf891c0c0c080b8412a6649625d0a5d11b7a075763015324b2b343924cf96a30eb173e3f097f8b9c24fc12a88c84d1a9cd7aa38b9384cd0e38b567405f31189b546f4d0098f0a2d1000c3808080f8440fb8401d2a8c9d328d7b1a4cb17c61e3650618bd79f1f00c073ad2998424277adf197c2c655040821e89a99934813b84b7823ef7a06045cbe58502af31a2744a35567e80";
        //let extraDleleteAanAggData2Test = await  lightClient._deleteSealAndAggTest(extra,extraDeleteAggTest);
        // console.log( extraDleleteAanAggData2Test)


        //console.log( await lightClient._decodeExtraData(extraData));
        //console.log(await lightClient.splitSignature("0x5113031cf7d57d9d111ac902b582bc855c029cab4d524de47cac0ae9127d0dd01ce8ef84d20b3465cea5d99c3889103ab7ec2c7b11447762a1920d6f1838147b01"))
        //console.log(await lightClient._deleteAgg(extra,extraData));
        //console.log(await lightClient._deleteSealAndAgg(extra,extraData))


        //header[10] = extraDeleteAgg;


        //console.log( header);
        // headerInfo.extraData = extraDleleteAanAggData1
        // console.log(headerInfo)
        //let header1 =
        // [
        //     '0x4d38359349eb407e28bd4e985010506ba934996102b5551492836016fc749b49',
        //     '0x6C5938B49bACDe73a8Db7C3A7DA208846898BFf5',
        //     '0xbe0461a71dfa948ab0a94fb034cfb6fce599dca83044d4f104466b743061a9ef',
        //     '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
        //     '0x6d013934131ee6c51393a8648672ec3225fb412478b83a1dd53f73ef545b4439',
        //     '0x10040000000000000000000000000000000000000000000000000000001020010000000080800000000000000000000000000000000008000000000004000000000000000004000000000008200000000020000000000000000000000000000000000000020000000000000000000800000000100a0000000000001000000800000000002000000000080000000000000000000000000000000020000000000004000000000000008000200000008040000000000080000000080000000000000000000a000002000000000000000000000080000000000000200000080020000400004000000000008100000000000000000000000000004000000000000000',
        //     '1000',
        //     '8000000',
        //     '0',
        //     '1653286079',
        //     extraDleleteAanAggData2Test,
        //     '0x0000000000000000000000000000000000000000000000000000000000000000',
        //     '0x0000000000000000',
        //     '100000000000'
        // ]
        // let header1 = [
        //         '0x9a2c09dc9f15e67f86dbf339e148ba0b4d0170fbfb72e420e30eaae1604b6669',
        //         '0xF18D71e825C43e5Ee5F3bD0384670eEf53a3309e',
        //         '0x83c411e2b84bfdf0fd682b97b8b76907c8ae0dd7fb6f5dff9790dd1fbce5ddb3',
        //         '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
        //         '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
        //         '0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
        //          "1" ,
        //          "19980470" ,
        //          "0" ,
        //       "1639123006" ,
        //         extraDleleteAanAggData2,
        //         '0x0000000000000000000000000000000000000000000000000000000000000000',
        //         '0x0000000000000000',
        //       "875000000" ,
        // ]
        // console.log( header1);
        //
        // let hashHeader = await lightClient._encodeHeader(header1);
        //console.log(hashHeader);



        // let blockInfo = await lightClient._encodeSealHash(
        //     "2145638",
        //     "0xe1c1aede3f35eb0d7028a48624528058516f2d86fe3e9c8e936d640f3efc09c1",
        //     "0xA47444C9dAAC489777dfEB5f30b03A6F3B4b6337"
        //
        // );


        //console.log("headerHash:" +headerHash);

        // let dataHash5 = await lightClient.getHash(hashHeader);
        // console.log("hash3------" + dataHash5.hash3);
        // console.log("hash1------" + dataHash5.hash1);

        //console.log("seal:" + extra.seal)
        // console.log(await lightClient._verifySign(extra.seal,dataHash5.hash3,"0x6C5938B49bACDe73a8Db7C3A7DA208846898BFf5"));
        // console.log(await lightClient._verifySign(extra.seal,dataHash5.hash1,"0x6C5938B49bACDe73a8Db7C3A7DA208846898BFf5"));



    });

    it('_verifyHeader ', async function () {
        // console.log(headerHash)
        console.log(await lightClient._verifyHeader(headerHash));

    });



});
