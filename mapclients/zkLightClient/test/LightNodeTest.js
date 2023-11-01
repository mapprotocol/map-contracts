const { ethers } = require("hardhat");
const proofs = require("./data");
const { expect } = require("chai");
require("solidity-coverage");

describe("LightNode start test", function () {
    const maxValidatorsLength = 32 * 5 * 128;
    let addr1;

    let LightClient;
    let lightClient;
    let lightNodeContract;
    let lightNodeContractAddress;

    let VerifyToolClient;
    let verifyToolClient;
    let verifyToolContract;
    let verifyToolContractAddress;

    let zkVerifierAddress;

    let LightProxyClient;
    let lightProxyClient;
    let proxy;

    let blsCode;
    let bc;

    let initValidatorsInfo;

    let datInit;

    beforeEach(async function () {
        [owner, addr1] = await ethers.getSigners();
        VerifyToolClient = await ethers.getContractFactory("VerifyTool");
        verifyToolClient = await VerifyToolClient.deploy();
        verifyToolContract = await verifyToolClient.deployed();
        verifyToolContractAddress = verifyToolContract.address;

        //deploy zkVerifier
        let zkvf = await ethers.getContractFactory("Verifier");
        zkvf = await zkvf.deploy();
        zkvc = await zkvf.deployed();
        zkVerifierAddress = zkvc.address;
    });

    it("deploy LightNode", async function () {
        LightClient = await ethers.getContractFactory("LightNode");
        lightClient = await LightClient.deploy();
        lightNodeContract = await lightClient.deployed();
        lightNodeContractAddress = lightNodeContract.address;
    });

    it("initialize ", async function () {
        let g2Hex = [
            "0x19587d8e318b681e8ff036af9c296b321f5066d03180705c4b244fd0555fcb6c0a4cdba7fb21dfba83a727ec4038967366af667efa211fe561a9ae53182040771189dcfaede47716b3e34cd3eea96dd791ca7947a62fb53721ad0ddc0dd15bcb2c41d5d6281528141ecbbbf836891742c9ac480165366bcc8ec79814f306aab7",
            "0x262292118774c85cb53e158bd3938ea804d7e9b28fa5a49ccbd23d18fb9a789b1dcd8922043d6563298355442be7680027cc0df3aa675a82d563c56db7c3903817e2042ceaa101dfbf5b279efe0dfb3842faa278486b1aa82202f92c6a6ec0a826ce3f717b3d1fd483ab0af83bc98fbc03caa7a97f32383100b48508a2b412ba",
            "0x14f821e131ea6c273607d704a4458408776354c2451ede5ba2c1e5195ec3929a109cc2d5a9487bf1a0a7b498678da54284509df5a2f74e03f8965707273dc4c70a6f29acb7b49f2e9f629c410a0923b516d23f3858611c500304a8f5c80454140ab032a87cf62167d7f16b05d7ad644a3862d1b9bb08e3baa9c39798ba1f0efa",
            "0x1acabcd4e2298c91e1752620a9e99e9c49513eaebde1aad381bf2946a40214f0071c72a02e910cf0318f954e3d09588af4d0ea5fec65a0c6511b0cb6423dc76d06e0d787fac2b683fa6f19f734dfbdd8bde1cb92924695679d81e265cded37cd07a3abe53e82f957db92a4dc279a3b7a5591c775fc1fb79a8185d95bbb93b2bb",
            "0x23378edd46da93002a90a525145d47cb834c53668494d3f8cf36294541baebcb0d1e3312fe248ccd541e561a34b9404ea1d70f2a21831b09f3d0ddf4ed923f5509c90688826a862a0d0394ae9bdd3e904300a0c08b7aea3b49c83e5ee8b3dea409bca6662f6a04bf928cb5fb2a55191e8b7fd9b5db1da3c876480f5427770ee0",
        ];
        let weights = [
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ];
        initValidatorsInfo = "0x" + g2Hex.map((item, index) => item.slice(2) + weights[index].slice(2)).join("");
        let padLength = maxValidatorsLength - g2Hex.length * 32 * 5;
        initValidatorsInfo = initValidatorsInfo + "00".repeat(padLength);

        blsCode = await ethers.getContractFactory("BlsCode");
        bc = await blsCode.deploy();
        await bc.deployed();

        let addresss = [
            "0x053af2b1ccbacba47c659b977e93571c89c49654",
            "0xb47adf1e504601ff7682b68ba7990410b92cd958",
            "0xf655fc7c95c70a118f98b46ca5028746284349a5",
            "0xb243f68e8e3245464d21b79c7ceae347ecc08ea6",
            "0x98efa292822eb7b3045c491e8ae4e82b3b1ac005",
        ];

        let _epoch = 203;

        let _epochSize = 1000;

        let data = await lightClient.initialize(
            initValidatorsInfo,
            addresss.length,
            _epoch,
            _epochSize,
            verifyToolContractAddress,
            zkVerifierAddress
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
        await proxy.updateBlockHeader(initValidatorsInfo, proofs.header203000, proofs.ist203000, proofs.zkProof20300);

        await proxy.updateBlockHeader(
            proofs.curValidatorInfo204000 + "00".repeat(120 * 32 * 5),
            proofs.header204000,
            proofs.ist204000,
            proofs.zkProof204000
        );

        await proxy.updateBlockHeader(
            proofs.curValidatorInfo205000 + "00".repeat(120 * 32 * 5),
            proofs.header205000,
            proofs.ist205000,
            proofs.zkProof205000
        );

        let prove205030 = await proxy.getBytes(proofs.provedata205030, proofs.zkProof205030);

        //let data205030 = await proxy.callStatic.verifyProofData(

        //   await proxy.getBytes(proofs.provedata205030),
        //   proofs.zkProof205030
        // );
        let data205030 = await proxy.callStatic.verifyProofData(prove205030);

        expect(data205030.success).to.equal(true);
    });

    it("add validator", async function () {
        await proxy.updateBlockHeader(
            proofs.curValidatorInfo206000 + "00".repeat(120 * 32 * 5),
            proofs.header206000,
            proofs.ist206000,
            proofs.zkProof206000
        );
        let data206460 = await proxy.callStatic.verifyProofData(
            await proxy.getBytes(proofs.provedata206460, proofs.zkProof206460)
        );
        expect(data206460.success).to.equal(true);
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

        await proxy.updateBlockHeader(
            proofs.curValidatorInfo207000 + "00".repeat(120 * 32 * 5),
            proofs.header207000,
            proofs.ist207000,
            proofs.zkProof207000
        );

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
        let g2Hex = [
            "0x19587d8e318b681e8ff036af9c296b321f5066d03180705c4b244fd0555fcb6c0a4cdba7fb21dfba83a727ec4038967366af667efa211fe561a9ae53182040771189dcfaede47716b3e34cd3eea96dd791ca7947a62fb53721ad0ddc0dd15bcb2c41d5d6281528141ecbbbf836891742c9ac480165366bcc8ec79814f306aab7",
            "0x262292118774c85cb53e158bd3938ea804d7e9b28fa5a49ccbd23d18fb9a789b1dcd8922043d6563298355442be7680027cc0df3aa675a82d563c56db7c3903817e2042ceaa101dfbf5b279efe0dfb3842faa278486b1aa82202f92c6a6ec0a826ce3f717b3d1fd483ab0af83bc98fbc03caa7a97f32383100b48508a2b412ba",
            "0x14f821e131ea6c273607d704a4458408776354c2451ede5ba2c1e5195ec3929a109cc2d5a9487bf1a0a7b498678da54284509df5a2f74e03f8965707273dc4c70a6f29acb7b49f2e9f629c410a0923b516d23f3858611c500304a8f5c80454140ab032a87cf62167d7f16b05d7ad644a3862d1b9bb08e3baa9c39798ba1f0efa",
            "0x1acabcd4e2298c91e1752620a9e99e9c49513eaebde1aad381bf2946a40214f0071c72a02e910cf0318f954e3d09588af4d0ea5fec65a0c6511b0cb6423dc76d06e0d787fac2b683fa6f19f734dfbdd8bde1cb92924695679d81e265cded37cd07a3abe53e82f957db92a4dc279a3b7a5591c775fc1fb79a8185d95bbb93b2bb",
            "0x23378edd46da93002a90a525145d47cb834c53668494d3f8cf36294541baebcb0d1e3312fe248ccd541e561a34b9404ea1d70f2a21831b09f3d0ddf4ed923f5509c90688826a862a0d0394ae9bdd3e904300a0c08b7aea3b49c83e5ee8b3dea409bca6662f6a04bf928cb5fb2a55191e8b7fd9b5db1da3c876480f5427770ee0",
        ];
        let weights = [
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ];
        let initValidatorsInfo = "0x" + g2Hex.map((item, index) => item.slice(2) + weights[index].slice(2)).join("");
        let padLength = maxValidatorsLength - g2Hex.length * 32 * 5;
        initValidatorsInfo = initValidatorsInfo + "00".repeat(padLength);

        let addresss = [
            "0x053af2b1ccbacba47c659b977e93571c89c49654",
            "0xb47adf1e504601ff7682b68ba7990410b92cd958",
            "0xf655fc7c95c70a118f98b46ca5028746284349a5",
            "0xb243f68e8e3245464d21b79c7ceae347ecc08ea6",
            "0x98efa292822eb7b3045c491e8ae4e82b3b1ac005",
        ];

        let _epoch = 217;

        let _epochSize = 1000;

        await lightClientDelete.initialize(
            initValidatorsInfo,
            addresss.length,
            _epoch,
            _epochSize,
            verifyToolContractAddress,
            zkVerifierAddress
        );

        await lightClientDelete.updateBlockHeader(
            initValidatorsInfo,
            proofs.header217000,
            proofs.ist217000,
            proofs.zkProof217000
        );

        await lightClientDelete.updateBlockHeader(
            proofs.curValidatorInfo218000 + "00".repeat(120 * 32 * 5),
            proofs.header218000,
            proofs.ist218000,
            proofs.zkProof218000
        );

        await lightClientDelete.updateBlockHeader(
            proofs.curValidatorInfo219000 + "00".repeat(120 * 32 * 5),
            proofs.header219000,
            proofs.ist219000,
            proofs.zkProof219000
        );

        let data220558 = await lightClientDelete.callStatic.verifyProofData(
            await lightClientDelete.getBytes(proofs.provedata220559, proofs.zkProof220559)
        );
        expect(data220558.success).to.equal(false);
        expect(data220558.message).to.equal("LightNode: header height error");

        await lightClientDelete.updateBlockHeader(
            proofs.curValidatorInfo220000 + "00".repeat(120 * 32 * 5),
            proofs.header220000,
            proofs.ist220000,
            proofs.zkProof220000
        );

        let data220559 = await lightClientDelete.callStatic.verifyProofData(
            await lightClientDelete.getBytes(proofs.provedata220559, proofs.zkProof220559)
        );
        //console.log(await lightClientDelete.getBytes(proofs.provedata220559, proofs.zkProof220559));
        expect(data220559.success).to.equal(true);

        let dataErr = await lightClientDelete.callStatic.verifyProofData(
            await lightClientDelete.getBytes(proofs.provedataHeaderError, proofs.zkProof220559)
            // Incorrect zk proofs
        );
        expect(dataErr.message).to.equal("LightNode: verifyHeaderSig fail");
    });
});
