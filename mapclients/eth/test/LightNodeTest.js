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

    let g1List;

    let dataInit;

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

        g1List = proofs.g1Init;

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
            verifyToolContractAddress,
            owner.address
        );
        dataInit = data.data;
    });

    it("deploy LightNodeProxy", async function () {
        LightProxyClient = await ethers.getContractFactory("LightNodeProxy");
        lightProxyClient = await LightProxyClient.deploy(lightNodeContractAddress, dataInit);
        await lightProxyClient.deployed();

        proxy = LightClient.attach(lightProxyClient.address);
    });

    it("updateBlockHeader and verifyProofData", async function () {
        console.log("update header 203000 ...");
        await proxy.updateBlockHeader(proofs.header203000, proofs.ist203000, proofs.aggpk203000);
        console.log("update header 204000 ...");
        await proxy.updateBlockHeader(proofs.header204000, proofs.ist204000, proofs.aggpk204000);
        console.log("update header 205000 ...");
        await proxy.updateBlockHeader(proofs.header205000, proofs.ist205000, proofs.aggpk205000);

        console.log(await proxy.headerHeight());

        await proxy["verifyProofDataWithCache(bytes)"](await proxy.getBytes(proofs.provedata205030));

        //console.log(await proxy.getValidators(205))
        let data205030 = await proxy["verifyProofData(bytes)"](await proxy.getBytes(proofs.provedata205030));
        expect(data205030.success).to.equal(true);

        expect(await proxy.isCachedReceiptRoot(205030)).to.be.equal(true)
    });

    it("add validator", async function () {
        await proxy.updateBlockHeader(proofs.header206000, proofs.ist206000, proofs.aggpk206000);

        let provedata206460Bytes = await proxy.getBytes(proofs.provedata206460);
        //console.log(provedata206460Bytes)
        //await proxy.verifyProofDataWithCache(provedata206460Bytes)
        await proxy["verifyProofDataWithCache(bool,uint256,bytes)"](true,0,provedata206460Bytes)

        let data206460 = await proxy["verifyProofData(bytes)"](await proxy.getBytes(proofs.provedata206460));
        expect(data206460.success).to.equal(true);

        expect(await proxy.isCachedReceiptRoot(206460)).to.be.equal(true)

    });

    it("authorizeUpgrade test ", async function () {
        const LightClientP1 = await ethers.getContractFactory("LightNode");
        let lightClientP1 = await LightClientP1.deploy();
        await lightClientP1.deployed();

        await proxy.upgradeTo(lightClientP1.address);

        expect(await proxy.getImplementation()).to.equal(lightClientP1.address);

        await proxy.setPendingAdmin(addr1.address);

        await (await proxy.connect(addr1)).changeAdmin();

        expect(await proxy.getAdmin()).to.equal(addr1.address);

        await proxy.updateBlockHeader(proofs.header207000, proofs.ist207000, proofs.aggpk207000);

        expect(await proxy.headerHeight()).to.equal("207000");
    });

    let LightClientDelete;
    let lightClientDelete;
    let lightNodeContractDelete;
    let lightNodeContractDeleteAddress;

    it("delete deploy", async function () {
        LightClientDelete = await ethers.getContractFactory("LightNode");
        lightClientDelete = await LightClientDelete.deploy();
        lightNodeContractDelete = await lightClientDelete.deployed();
        lightNodeContractDeleteAddress = lightNodeContractDelete.address;
    });

    it("verifyProofData error test ", async function () {

        let g1ListDelete = proofs.g1Init;

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
            verifyToolContractAddress,
            owner.address
        );

        await lightClientDelete.updateBlockHeader(proofs.header217000, proofs.ist217000, proofs.aggpk217000);
        await lightClientDelete.updateBlockHeader(proofs.header218000, proofs.ist218000, proofs.aggpk218000);
        await lightClientDelete.updateBlockHeader(proofs.header219000, proofs.ist219000, proofs.aggpk219000);

        let data220558 = await lightClientDelete["verifyProofData(bytes)"](
            await lightClientDelete.getBytes(proofs.provedata220559)
        );
        expect(data220558.success).to.equal(false);
        expect(data220558.message).to.equal("Out of verify range");
        await lightClientDelete.updateBlockHeader(proofs.header220000, proofs.ist220000, proofs.aggpk220000);

        let data220559 =  await lightClientDelete["verifyProofData(bytes)"]( await lightClientDelete.getBytes(proofs.provedata220559));
        expect(data220559.success).to.equal(true);

        let dataErr = await lightClientDelete["verifyProofData(bytes)"](
            await lightClientDelete.getBytes(proofs.provedataHeaderError)
        );
        expect(dataErr.message).to.equal("VerifyHeaderSig failed");
    });
});
