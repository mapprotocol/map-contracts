const { ethers } = require("hardhat");
const { expect } = require("chai");
const mosRelayData = require('./mosRelayData');
require("solidity-coverage");


describe("MAPOmnichainServiceRelayV2 start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;
    let addr6;
    let addr7;
    let addr8;

    let MOSSRelay;
    let mossR;

    let StandardToken;
    let standardToken;

    let UToken;
    let usdt;


    let LightClientManager;
    let lightClientManager;

    let address2Bytes;

    beforeEach(async function () {

        [deployer,owner, addr1, addr2, addr3, addr4, addr5,addr6,addr7,addr8] = await ethers.getSigners();

    });

    it("MAPOmnichainServiceRelayV2 contract deploy init", async function () {
        console.log("deployer address:",deployer.address)

        StandardToken = await ethers.getContractFactory("MintableToken");
        standardToken = await  StandardToken.deploy("MapToken","MP");

        UToken = await ethers.getContractFactory("MintableToken");
        usdt = await  UToken.deploy("U Toeken","USDT");

        LightClientManager = await ethers.getContractFactory("LightClientManager");
        lightClientManager = await LightClientManager.deploy();
        console.log("LightClientManager   address:",lightClientManager.address);

        MOSSRelay = await ethers.getContractFactory("MAPOmnichainServiceRelayV2");
        // moss = await ethers.getContractAt("MapCrossChainService",mosData.mos);
        mossR = await MOSSRelay.deploy(lightClientManager.address);
        console.log("mossR address:",mossR.address);

    });

    it('mosRelay contract set ', async function () {
        await mossR.regToken(mosRelayData.ethUsdtToken,usdt.address);
        await mossR.regToken(mosRelayData.ethStanardToken,standardToken.address);
    });


    it('transferOutToken', async function () {

        address2Bytes = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";

        await standardToken.mint(owner.address,"100000000000000000000");

        await standardToken.connect(owner).approve(mossR.address,"100000000000000000000");


        await mossR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"100000000000000000000",5)

        expect(await standardToken.totalSupply()).to.equal("100000000000000000000");

        expect(await standardToken.balanceOf(owner.address)).to.equal("0");
    });


    it('eth2map transferIn test', async function () {
        expect(await usdt.balanceOf(mossR.address)).to.equal("0");
        await usdt.mint(mossR.address,"100000000000000000000");
        await mossR.transferIn(97,mosRelayData.eth2mapMapToken);
        expect(await usdt.balanceOf(mossR.address)).to.equal("0")

        await standardToken.mint(mossR.address,"200000000000000000000")
        await mossR.transferIn(97,mosRelayData.eth2mapStandardToken);
        expect(await standardToken.totalSupply()).to.equal("300000000000000000000");

        expect(await standardToken.balanceOf(mossR.address)).to.equal("0");
    });



})
