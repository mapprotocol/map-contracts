const { ethers } = require("hardhat");
const headers = require('./data');
const {expect} = require('chai');
const Caver = require('caver-js')
require("solidity-coverage");
const { BigNumber} = require("ethers");

describe("LightNode start test", function () {
    let lightNodeContract;
    let lightNodeContractAddress;
    let LightNodeProxy;
    let verifyTool;
    let mpt;
    let owner;
    let adminChange;
    let caver;
    let startHeight = 108288000;

    beforeEach(async function () {
        [owner, adminChange] = await ethers.getSigners();
        caver = new Caver("https://public-node-api.klaytnapi.com/v1/baobab");
        //caver = new Caver("https://public-node-api.klaytnapi.com/v1/cypress");
    });

    it("deploy LightNode",async function () {
        let LightClient = await ethers.getContractFactory("LightNode");

        let lightClient = await LightClient.deploy();

        lightNodeContract = await lightClient.deployed()

        lightNodeContractAddress = lightNodeContract.address;
    });


    it("deploy LightNode Proxy",async function () {

        let height = startHeight;

        console.log("init height:",height);

        let block = await caver.rpc.klay.getBlockByNumber(height);

        verifyTool = await (await ethers.getContractFactory("VerifyTool")).deploy();

        mpt = await (await ethers.getContractFactory("MPTVerify")).deploy();

        let result = await verifyTool.decodeHeaderExtraData(block.extraData);

        let data = lightNodeContract.interface.encodeFunctionData("initialize",
            [result.extData.validators,block.number,verifyTool.address,mpt.address]);

        console.log("validators",result.extData.validators)

        let proxy = await ethers.getContractFactory("LightNodeProxy");

        let lightProxyClient = await proxy.deploy(lightNodeContractAddress,data);

        await lightProxyClient.deployed()

        LightNodeProxy = lightNodeContract.attach(lightProxyClient.address);

        console.log("LightNode Proxy deploy ok")
    });


    it("lightNodeContract params check", async function (){
        let heightHeight = await LightNodeProxy.headerHeight();

        heightHeight =  ethers.utils.formatUnits(heightHeight,0)

        expect(heightHeight).to.equal(startHeight.toString())

        let range = await LightNodeProxy.verifiableHeaderRange();

        let end = ethers.utils.formatUnits(range.end,0)

        expect(end).to.equal((startHeight + 3600).toString())
    });


    it("lightNode verify Header",async function (){

        let result = await LightNodeProxy.verifyProofData(headers.verifyProof);
        expect(result.success).eq(true);
    });



    it("lightNode updateBlockHeader", async function (){
        startHeight = startHeight + 3600;
        let block = await caver.rpc.klay.getBlockByNumber(startHeight);
        let header = [];
        header.push(block.parentHash)
        header.push(block.reward)
        header.push(block.stateRoot)
        header.push(block.transactionsRoot)
        header.push(block.receiptsRoot)
        header.push(block.logsBloom)
        header.push(block.blockScore)
        header.push(block.number)
        header.push(block.gasUsed)
        header.push(block.timestamp)
        header.push(block.timestampFoS)
        header.push(block.extraData)
        header.push(block.governanceData)
        header.push(block.voteData)
        header.push(block.baseFeePerGas)
        let headers = [header];



        let headerBytes = await LightNodeProxy.getHeadersBytes(headers);

        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((108288000+ 3600).toString()).to.eq(heightHeight);
    });


    it("lightNode updateBlockHeaders", async function (){
        let headers = [];
        for (i=0;i<20;i++){
            startHeight = startHeight + 3600;

            let block = await caver.rpc.klay.getBlockByNumber(startHeight);
            let header = [];
            header.push(block.parentHash)
            header.push(block.reward)
            header.push(block.stateRoot)
            header.push(block.transactionsRoot)
            header.push(block.receiptsRoot)
            header.push(block.logsBloom)
            header.push(block.blockScore)
            header.push(block.number)
            header.push(block.gasUsed)
            header.push(block.timestamp)
            header.push(block.timestampFoS)
            header.push(block.extraData)
            header.push(block.governanceData)
            header.push(block.voteData)
            header.push(block.baseFeePerGas)
            headers.push(header);
        }

        let headerBytes = await LightNodeProxy.getHeadersBytes(headers);

        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((108288000+ 3600 * 21).toString()).to.eq(heightHeight);
    });


    it("lightNode admin Test",async function (){
        let admin = await LightNodeProxy.owner();

        expect(admin).to.equal(owner.address);

        await LightNodeProxy.transferOwnership(adminChange.address);

        admin = await LightNodeProxy.owner();

        expect(admin).to.equal(owner.address);

        //await expect(LightNodeProxy.acceptOwnership()).to.be.revertedWith("Ownable2Step: caller is not the new owner");

        await LightNodeProxy.connect(adminChange).acceptOwnership()

        admin = await LightNodeProxy.owner();

        expect(admin).to.equal(adminChange.address);

        let uupsAdmin = await LightNodeProxy.getAdmin();

        expect(uupsAdmin).to.equal(adminChange.address);

        await LightNodeProxy.connect(adminChange).transferOwnership(owner.address);

        await LightNodeProxy.acceptOwnership();

    });


    let startHeightRamify = 114267600;

    it("deploy LightNode 111736800",async function () {
        let LightClient = await ethers.getContractFactory("LightNode");

        let lightClient = await LightClient.deploy();

        lightNodeContract = await lightClient.deployed()

        lightNodeContractAddress = lightNodeContract.address;
    });


    it("deploy LightNode Proxy 111736800",async function () {

        let height = startHeightRamify;

        console.log("init height:",height);

        let block = await caver.rpc.klay.getBlockByNumber(height);


        let result = await verifyTool.decodeHeaderExtraData(block.extraData);

        let data = lightNodeContract.interface.encodeFunctionData("initialize",
            [result.extData.validators,block.number,verifyTool.address,mpt.address]);

        console.log("validators",result.extData.validators)

        let proxy = await ethers.getContractFactory("LightNodeProxy");

        let lightProxyClient = await proxy.deploy(lightNodeContractAddress,data);

        await lightProxyClient.deployed()

        LightNodeProxy = lightNodeContract.attach(lightProxyClient.address);

        console.log("LightNode Proxy deploy ok")
    });

    it("lightNode verify proof 111736800",async function (){

        let result = await LightNodeProxy.verifyProofData(headers.transcationProof1);
        expect(result.success).eq(true);
    });


    it("lightNode updateBlockHeaders", async function (){
        let headers = [];
        for (i=0;i<22;i++){
            startHeightRamify = startHeightRamify + 3600;

            let block = await caver.rpc.klay.getBlockByNumber(startHeightRamify);
            let header = [];
            header.push(block.parentHash)
            header.push(block.reward)
            header.push(block.stateRoot)
            header.push(block.transactionsRoot)
            header.push(block.receiptsRoot)
            header.push(block.logsBloom)
            header.push(block.blockScore)
            header.push(block.number)
            header.push(block.gasUsed)
            header.push(block.timestamp)
            header.push(block.timestampFoS)
            header.push(block.extraData)
            header.push(block.governanceData)
            header.push(block.voteData)
            header.push(block.baseFeePerGas)
            headers.push(header);
        }

        let headerBytes = await LightNodeProxy.getHeadersBytes(headers);

        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((114267600+ 3600 * 22).toString()).to.eq(heightHeight);
    });

    it("lightNode verify proof 114350210",async function (){

        let result = await LightNodeProxy.verifyProofData(headers.transcationProof2);

        expect(result.success).eq(true);
    });


});
