const { ethers } = require("hardhat");
const proofs = require("./data");
const { expect } = require("chai");
require("solidity-coverage");

describe("LightNode start test", function () {
    let owner;
    let addr1;

    let LightClient;
    let lightClient;
    let lightNodeContract;
    let lightNodeContractAddress;

    let VerifyToolClient;
    let verifyToolClient;
    let verifyToolContract;
    let verifyToolContractAddress;

    let LightProxyClient;
    let lightProxyClient;
    let proxy;

    let blsCode;
    let bc;

    let g1List;

    let datInit;

    beforeEach(async function () {
        [owner, addr1] = await ethers.getSigners();
        VerifyToolClient = await ethers.getContractFactory("VerifyTool");
        verifyToolClient = await VerifyToolClient.deploy();
        verifyToolContract = await verifyToolClient.deployed();
        verifyToolContractAddress = verifyToolContract.address;
    });

    it("deploy LightNode", async function () {
        LightClient = await ethers.getContractFactory("LightNode");
        lightClient = await LightClient.deploy();
        lightNodeContract = await lightClient.deployed();
        lightNodeContractAddress = lightNodeContract.address;
    });

    it("initialize ", async function () {
        let g1Hex = [
            "0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c12b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287",
            "0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a692685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7",
            "0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042",
            "0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37",
            "0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116",
        ];

        //blsCode = await ethers.getContractFactory("BlsCode");
        //bc = await blsCode.deploy();
        //await bc.deployed();

        //const g0 = await bc.decodeG1(g1Hex[0]);
        //const g1 = await bc.decodeG1(g1Hex[1]);
        //const g2 = await bc.decodeG1(g1Hex[2]);
        //const g3 = await bc.decodeG1(g1Hex[3]);
        //const g4 = await bc.decodeG1(g1Hex[4]);

        const g0 = [
            "0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c1",
            "0x2b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287",
        ];
        const g1 = [
            "0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a69",
            "0x2685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7",
        ];
        const g2 = [
            "0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835",
            "0x156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042",
        ];
        const g3 = [
            "0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b",
            "0x131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37",
        ];
        const g4 = [
            "0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a",
            "0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116",
        ];

        g1List = [g0, g1, g2, g3, g4];

        let addresss = [
            "0x053af2b1ccbacba47c659b977e93571c89c49654",
            "0xb47adf1e504601ff7682b68ba7990410b92cd958",
            "0xf655fc7c95c70a118f98b46ca5028746284349a5",
            "0xb243f68e8e3245464d21b79c7ceae347ecc08ea6",
            "0x98efa292822eb7b3045c491e8ae4e82b3b1ac005",
        ];
        let _weights = [1, 1, 1, 1, 1];

        let _threshold = 3;

        let _epoch = 203;

        let _epochSize = 1000;

        let data = await lightClient.initialize(
            _threshold,
            addresss,
            g1List,
            _weights,
            _epoch,
            _epochSize,
            verifyToolContractAddress
        );
        datInit = data.data;
    });

    it("deploy LightNodeProxy", async function () {
        LightProxyClient = await ethers.getContractFactory("LightNodeProxy");
        lightProxyClient = await LightProxyClient.deploy(lightNodeContractAddress, datInit);
        await lightProxyClient.deployed();

        proxy = LightClient.attach(lightProxyClient.address);
    });

    it("updateBlockHeader and verifyProofData", async function () {
        await proxy.updateBlockHeader(proofs.header203000, proofs.ist203000, proofs.aggpk203000);
        await proxy.updateBlockHeader(proofs.header204000, proofs.ist204000, proofs.aggpk204000);
        await proxy.updateBlockHeader(proofs.header205000, proofs.ist205000, proofs.aggpk205000);

        let data205030 = await proxy.callStatic.verifyProofData(await proxy.getBytes(proofs.provedata205030));
        expect(data205030.success).to.equal(true);
    });

    it("add validator", async function () {
        await proxy.updateBlockHeader(proofs.header206000, proofs.ist206000, proofs.aggpk206000);

        let data206460 = await proxy.callStatic.verifyProofData(await proxy.getBytes(proofs.provedata206460));
        expect(data206460.success).to.equal(true);
    });

    it("authorizeUpgrade test ", async function () {
        const LightClientP1 = await ethers.getContractFactory("LightNode");
        let lightClientP1 = await LightClientP1.deploy();
        await lightClientP1.deployed();

        await proxy.upgradeTo(lightClientP1.address);

        expect(await proxy.getImplementation()).to.equal(lightClientP1.address);

        await proxy.setPendingAdmin(addr1.address);
        // console.log(addr1.address);
        // console.log(await proxy.pendingAdmin());

        await (await proxy.connect(addr1)).changeAdmin();

        expect(await proxy.getAdmin()).to.equal(addr1.address);

        await proxy.updateBlockHeader(proofs.header207000, proofs.ist207000, proofs.aggpk207000);

        expect(await proxy.headerHeight()).to.equal("207000");
    });

    let LightClientDelete;
    let lightClientDelete;
    let lightNodeContractDelete;
    let lightNodeContractDeleteAddress;

    let blsCodeDelete;
    let bcDelete;

    it("delete deploy", async function () {
        LightClientDelete = await ethers.getContractFactory("LightNode");
        lightClientDelete = await LightClientDelete.deploy();
        lightNodeContractDelete = await lightClientDelete.deployed();
        lightNodeContractDeleteAddress = lightNodeContractDelete.address;
        blsCodeDelete = await ethers.getContractFactory("BlsCode");
        bcDelete = await blsCodeDelete.deploy();
        await bcDelete.deployed();
    });

    it("verifyProofData error test ", async function () {
        let g1Hex = [
            "0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c12b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287",
            "0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a692685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7",
            "0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042",
            "0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37",
            "0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116",
        ];

        //const g0 = await bcDelete.decodeG1(g1Hex[0]);
        //const g1 = await bcDelete.decodeG1(g1Hex[1]);
        //const g2 = await bcDelete.decodeG1(g1Hex[2]);
        //const g3 = await bcDelete.decodeG1(g1Hex[3]);
        //const g4 = await bcDelete.decodeG1(g1Hex[4]);

        const g0 = [
            "0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c1",
            "0x2b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287",
        ];
        const g1 = [
            "0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a69",
            "0x2685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7",
        ];
        const g2 = [
            "0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835",
            "0x156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042",
        ];
        const g3 = [
            "0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b",
            "0x131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37",
        ];
        const g4 = [
            "0x2b8a812d2e9ac7d6799b3ebad52a27402a31e89eb3f383be96314f3f3f0ead3a",
            "0x028250eedb4307d62696f8a1b235dc376682780fb69eb1b7c9403ee6608ad116",
        ];

        let g1ListDelete = [g0, g1, g2, g3, g4];

        let addresss = [
            "0x053af2b1ccbacba47c659b977e93571c89c49654",
            "0xb47adf1e504601ff7682b68ba7990410b92cd958",
            "0xf655fc7c95c70a118f98b46ca5028746284349a5",
            "0xb243f68e8e3245464d21b79c7ceae347ecc08ea6",
            "0x98efa292822eb7b3045c491e8ae4e82b3b1ac005",
        ];
        let _weights = [1, 1, 1, 1, 1];

        let _threshold = 3;

        let _epoch = 217;

        let _epochSize = 1000;

        await lightClientDelete.initialize(
            _threshold,
            addresss,
            g1ListDelete,
            _weights,
            _epoch,
            _epochSize,
            verifyToolContractAddress
        );

        await lightClientDelete.updateBlockHeader(proofs.header217000, proofs.ist217000, proofs.aggpk217000);
        await lightClientDelete.updateBlockHeader(proofs.header218000, proofs.ist218000, proofs.aggpk218000);
        await lightClientDelete.updateBlockHeader(proofs.header219000, proofs.ist219000, proofs.aggpk219000);

        let data220558 = await lightClientDelete.callStatic.verifyProofData(
            await lightClientDelete.getBytes(proofs.provedata220559)
        );
        expect(data220558.success).to.equal(false);
        expect(data220558.message).to.equal("Out of verify range");
        await lightClientDelete.updateBlockHeader(proofs.header220000, proofs.ist220000, proofs.aggpk220000);

        // let data220559 =  await lightClientDelete.callStatic.verifyProofData( await lightClientDelete.getBytes(proofs.provedata220559));
        // expect(data220559.success).to.equal(true);
        // await  expect( lightClientDelete.callStatic.verifyProofData( await lightClientDelete.getBytes(proofs.provedataProofError))).to.be.revertedWith("verifyTrieProof root node hash invalid");
        //expect(dataProofError.message).to.equal("bls error");
        let dataErr = await lightClientDelete.callStatic.verifyProofData(
            await lightClientDelete.getBytes(proofs.provedataHeaderError)
        );
        expect(dataErr.message).to.equal("VerifyHeaderSig failed");
    });
});
