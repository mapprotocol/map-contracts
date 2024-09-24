const { ethers } = require("hardhat");
const proofs = require("./dataV2");
const { expect } = require("chai");
require("solidity-coverage");

describe("LightNode V2 start test", function () {
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

    let g1List;

    let dataInit;

    beforeEach(async function () {
        [owner, addr1] = await ethers.getSigners();
        VerifyToolClient = await ethers.getContractFactory("VerifyToolV2");
        verifyToolClient = await VerifyToolClient.deploy();
        verifyToolContract = await verifyToolClient.deployed();
        verifyToolContractAddress = verifyToolContract.address;
    });

    it("deploy LightNodeV2", async function () {
        LightClient = await ethers.getContractFactory("LightNodeV2");
        lightClient = await LightClient.deploy();
        lightNodeContract = await lightClient.deployed();
        lightNodeContractAddress = lightNodeContract.address;
    });

    it("initialize ", async function () {
        g1List = proofs.g1InitV2;

        let _weights = [1, 1, 1, 1, 1];
        let _threshold = 3;
        let _epoch = 203;
        let _epochSize = 1000;

        let data = await lightClient.initialize(
            _threshold,
            g1List,
            _weights,
            _epoch,
            _epochSize,
            verifyToolContractAddress,
            owner.address
        );
        dataInit = data.data;
    });

    it("deploy LightNodeV2Proxy", async function () {
        LightProxyClient = await ethers.getContractFactory("LightNodeProxy");
        lightProxyClient = await LightProxyClient.deploy(lightNodeContractAddress, dataInit);
        await lightProxyClient.deployed();

        proxy = LightClient.attach(lightProxyClient.address);
    });

    it("updateBlockHeader and verifyProofData", async function () {
        console.log("update header 203000 remove g4");
        await proxy.updateBlockHeader(
            "203000",
            proofs.deleteAggHeaderBytes203000,
            proofs.deleteSealAndAggHeaderBytes203000,
            proofs.ist203000,
            proofs.aggpk203000,
            g1List
        );
        g1List = proofs.g1ListV2;
        console.log("update header 204000 no update");
        await proxy.updateBlockHeader(
            "204000",
            proofs.deleteAggHeaderBytes204000,
            proofs.deleteSealAndAggHeaderBytes204000,
            proofs.ist204000,
            proofs.aggpk204000,
            g1List
        );

        console.log("update header 205000 np update");
        await proxy.updateBlockHeader(
            "205000",
            proofs.deleteAggHeaderBytes205000,
            proofs.deleteSealAndAggHeaderBytes205000,
            proofs.ist205000,
            proofs.aggpk205000,
            g1List
        );

        await proxy["verifyProofDataWithCache(bytes)"](await proxy.getBytes(proofs.provedataV2205030));

        //console.log(await proxy.newPairKeys())
        let data205030 = await proxy.verifyProofData(await proxy.getBytes(proofs.provedataV2205030));
        expect(data205030.success).to.equal(true);

        expect(await proxy.isCachedReceiptRoot(205030)).to.be.equal(true)
    });

    it("add validator", async function () {
        await proxy.updateBlockHeader(
            "206000",
            proofs.deleteAggHeaderBytes206000,
            proofs.deleteSealAndAggHeaderBytes206000,
            proofs.ist206000,
            proofs.aggpk206000,
            g1List
        );

        let provedata206460Bytes = await proxy.getBytes(proofs.provedataV2206460);
        console.log(provedata206460Bytes)

        await proxy["verifyProofDataWithCache(bool,uint256,bytes)"](true,0,provedata206460Bytes)

        let data206460 = await proxy.verifyProofData(await proxy.getBytes(proofs.provedataV2206460));
        expect(data206460.success).to.equal(true);

        expect(await proxy.isCachedReceiptRoot(206460)).to.be.equal(true)
    });

    it("authorizeUpgrade test ", async function () {
        const LightClientP1 = await ethers.getContractFactory("LightNodeV2");
        let lightClientP1 = await LightClientP1.deploy();
        await lightClientP1.deployed();

        await proxy.upgradeTo(lightClientP1.address);

        expect(await proxy.getImplementation()).to.equal(lightClientP1.address);

        await proxy.setPendingAdmin(addr1.address);

        await (await proxy.connect(addr1)).changeAdmin();

        expect(await proxy.getAdmin()).to.equal(addr1.address);
        g1List = proofs.g1InitV2;
        await proxy.updateBlockHeader(
            "207000",
            proofs.deleteAggHeaderBytes207000,
            proofs.deleteSealAndAggHeaderBytes207000,
            proofs.ist207000,
            proofs.aggpk207000,
            g1List
        );

        expect(await proxy.headerHeight()).to.equal("207000");
    });

    let LightClientDelete;
    let lightClientDelete;
    let lightNodeContractDelete;
    let lightNodeContractDeleteAddress;

    it("delete deploy", async function () {
        LightClientDelete = await ethers.getContractFactory("LightNodeV2");
        lightClientDelete = await LightClientDelete.deploy();
        lightNodeContractDelete = await lightClientDelete.deployed();
        lightNodeContractDeleteAddress = lightNodeContractDelete.address;
    });

    it("verifyProofData error test ", async function () {

        let g1ListDelete = proofs.g1InitV2;

        let _weights = [1, 1, 1, 1, 1];

        let _threshold = 3;

        let _epoch = 217;

        let _epochSize = 1000;

        await lightClientDelete.initialize(
            _threshold,
            g1ListDelete,
            _weights,
            _epoch,
            _epochSize,
            verifyToolContractAddress,
            owner.address
        );

        await lightClientDelete.updateBlockHeader(
            "217000",
            proofs.deleteAggHeaderBytes217000,
            proofs.deleteSealAndAggHeaderBytes217000,
            proofs.ist217000,
            proofs.aggpk217000,
            g1ListDelete
        );
        await lightClientDelete.updateBlockHeader(
            "218000",
            proofs.deleteAggHeaderBytes218000,
            proofs.deleteSealAndAggHeaderBytes218000,
            proofs.ist218000,
            proofs.aggpk218000,
            g1ListDelete
        );
        g1ListDelete = proofs.g1ListV2;
        await lightClientDelete.updateBlockHeader(
            "219000",
            proofs.deleteAggHeaderBytes219000,
            proofs.deleteSealAndAggHeaderBytes219000,
            proofs.ist219000,
            proofs.aggpk219000,
            g1ListDelete
        );

        await expect(lightClientDelete.verifyProofData(await lightClientDelete.getBytes(proofs.provedataV2220559)))
            .to.be.revertedWith("Out of verify range")

        await lightClientDelete.updateBlockHeader(
            "220000",
            proofs.deleteAggHeaderBytes220000,
            proofs.deleteSealAndAggHeaderBytes220000,
            proofs.ist220000,
            proofs.aggpk220000,
            g1ListDelete
        );

        await expect(lightClientDelete.verifyProofData(await lightClientDelete.getBytes(proofs.provedataHeaderErrorV2)))
            .to.be.revertedWith("keys hash error");
    });
});
