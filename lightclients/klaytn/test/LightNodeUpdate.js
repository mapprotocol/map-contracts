const { ethers } = require("hardhat");
const headers = require('./data');
const {expect} = require('chai');
const Caver = require('caver-js')
require("solidity-coverage");
const { BigNumber} = require("ethers");

describe("LightNode start test", function () {
    let lightNodeContract;
    let lightNodeContractAddress;
    let lightProxyClient;
    let LightNodeProxy;
    let verifyTool;
    let mpt;
    let owner;
    let adminChange;
    let caver;
    //let startHeight = 121017600 - 3600;
    let startHeight = 121006800;
    //121,014,000
    //121,010,400
    //121013234 REMOVE_VALIDATOR
    //121019147 ADD_VALIDATOR
    beforeEach(async function () {
        [owner, adminChange] = await ethers.getSigners();
        //caver = new Caver("https://public-node-api.klaytnapi.com/v1/baobab");
        //caver = new Caver("https://public-node-api.klaytnapi.com/v1/cypress");
        caver = new Caver("https://klaytn.blockpi.network/v1/rpc/public");
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

         lightProxyClient = await proxy.deploy(lightNodeContractAddress,data);

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

        expect(end).to.equal((startHeight + 3600 - 1).toString())
    });

    it("lightNode updateBlockHeaders", async function (){

        let headers = [];
        for (i=0;i<1;i++){
            startHeight =startHeight + 3600;
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

        //  await LightNodeProxy.updateBlockHeaderChange(headers);
        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((121010400).toString()).to.eq(heightHeight);
    });

    it('lightnode Update', async function () {
        let LightClientUp = await ethers.getContractFactory("LightNode");

        let lightClientUp = await LightClientUp.deploy();

        await lightClientUp.deployed()

        LightNodeProxy.upgradeTo(lightClientUp.address);

        LightNodeProxy = lightClientUp.attach(lightProxyClient.address);

    });

    //121013234 REMOVE_VALIDATOR
    it("lightNode updateBlockHeader REMOVE_VALIDATOR 121013234", async function (){

        let headers = [];
        startHeight = 121013234;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((121013235).toString()).to.eq(heightHeight);
    });

    //121,014,000
    it("lightNode updateBlockHeaders 121014000", async function (){

        startHeight =121010400;
        let headers = [];
        for (i=0;i<2;i++){
            startHeight += 3600;
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

        //  await LightNodeProxy.updateBlockHeaderChange(headers);
        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((121017600).toString()).to.eq(heightHeight);
    });

//121019147 ADD_VALIDATOR
    it("lightNode updateBlockHeader ADD_VALIDATOR 121019147", async function (){

        let headers = [];
        startHeight = 121019147;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((121019148).toString()).to.eq(heightHeight);
    });

    it("lightNode updateBlockHeaders 121021200", async function (){
        startHeight = 121017600;
        let headers = [];
        for (i=0;i<1;i++){
            startHeight += 3600;
            console.log(startHeight)
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

        //  await LightNodeProxy.updateBlockHeaderChange(headers);
        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((121021200).toString()).to.eq(heightHeight);
    });


    it("deploy LightNode",async function () {
        let LightClient = await ethers.getContractFactory("LightNode");

        let lightClient = await LightClient.deploy();

        lightNodeContract = await lightClient.deployed()

        lightNodeContractAddress = lightNodeContract.address;
    });


    it("deploy LightNode Proxy",async function () {

        let height = 134953200;

        console.log("init height:",height);

        let block = await caver.rpc.klay.getBlockByNumber(height);


        verifyTool = await (await ethers.getContractFactory("VerifyTool")).deploy();

        mpt = await (await ethers.getContractFactory("MPTVerify")).deploy();

        let result = await verifyTool.decodeHeaderExtraData(block.extraData);

        let data = lightNodeContract.interface.encodeFunctionData("initialize",
            [result.extData.validators,block.number,verifyTool.address,mpt.address]);

        console.log("validators",result.extData.validators)

        let proxy = await ethers.getContractFactory("LightNodeProxy");

        lightProxyClient = await proxy.deploy(lightNodeContractAddress,data);

        await lightProxyClient.deployed()

        LightNodeProxy = lightNodeContract.attach(lightProxyClient.address);

        console.log("LightNode Proxy deploy ok")
    });


    it("lightNode updateBlockHeader 134953471", async function (){

        let headers = [];
        startHeight = 134953471;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((134953472).toString()).to.eq(heightHeight);
    });

    it("lightNode updateBlockHeader REMOVE_VALIDATOR 134953529", async function (){

        let headers = [];
        startHeight = 134953529;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((134953530).toString()).to.eq(heightHeight);
    });

    it("lightNode updateBlockHeader ADD_VALIDATOR 134954063", async function (){

        let headers = [];
        startHeight = 134954063;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((134954064).toString()).to.eq(heightHeight);
    });

    it("lightNode updateBlockHeader 134954671", async function (){

        let headers = [];
        startHeight = 134954671;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((134954672).toString()).to.eq(heightHeight);
    });

    it("lightNode updateBlockHeaders 134953200", async function (){
        startHeight = 134953200;
        let headers = [];
        for (i=0;i<1;i++){
            startHeight += 3600;
            console.log(startHeight)
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

        //  await LightNodeProxy.updateBlockHeaderChange(headers);
        await LightNodeProxy.updateBlockHeader(headerBytes);

        let heightHeight = await LightNodeProxy.headerHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((134953200 + 3600).toString()).to.eq(heightHeight);
    });

    //135,021,600
    it("deploy LightNode",async function () {
        let LightClient = await ethers.getContractFactory("LightNode");

        let lightClient = await LightClient.deploy();

        lightNodeContract = await lightClient.deployed()

        lightNodeContractAddress = lightNodeContract.address;
    });


    it("deploy LightNode Proxy",async function () {

        let height = 135021600;

        console.log("init height:",height);

        let block = await caver.rpc.klay.getBlockByNumber(height);


        verifyTool = await (await ethers.getContractFactory("VerifyTool")).deploy();

        mpt = await (await ethers.getContractFactory("MPTVerify")).deploy();

        let result = await verifyTool.decodeHeaderExtraData(block.extraData);

        let data = lightNodeContract.interface.encodeFunctionData("initialize",
            [result.extData.validators,block.number,verifyTool.address,mpt.address]);

        console.log("validators",result.extData.validators)

        let proxy = await ethers.getContractFactory("LightNodeProxy");

        lightProxyClient = await proxy.deploy(lightNodeContractAddress,data);

        await lightProxyClient.deployed()

        LightNodeProxy = lightNodeContract.attach(lightProxyClient.address);

        console.log("LightNode Proxy deploy ok")
    });

    it("lightNode updateBlockHeader 135021967", async function (){

        let headers = [];
        startHeight = 135021967;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);

        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((135021968).toString()).to.eq(heightHeight);
    });

    it("lightNode updateBlockHeader REMOVE_VALIDATOR 135022022", async function (){

        let headers = [];
        startHeight = 135022022;
        for (i=0;i<2;i++){
            startHeight += i;
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

        let heightHeight = await LightNodeProxy.lastCommitteeHeight();
        console.log(heightHeight);



        heightHeight = ethers.utils.formatUnits(heightHeight,0)

        expect((135022023).toString()).to.eq(heightHeight);
    });

});
