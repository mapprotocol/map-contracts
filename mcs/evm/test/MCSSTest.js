const { ethers } = require("hardhat");
const { expect } = require("chai");
const mcsData = require('./mcsData');
require("solidity-coverage");
const { BigNumber, BytesLike, Contract, ContractTransaction } = require("ethers");
const {weiToHumanReadableString} = require("hardhat/internal/util/wei-values");

describe("MAPCrossChainServiceRelay start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;
    let addrs;

    let MCSS;
    let mcss;

    let StandardToken;
    let standardToken;

    let standardEthToken;
    let mapTokenEthToken;
    let wethEthToken;


    let address2Bytes;

    beforeEach(async function () {

        [deployer,owner, addr1, addr2, addr3, addr4, addr5,...addrs] = await ethers.getSigners();

    });

    it("constract deploy init", async function () {
        console.log("deployer address:",deployer.address);
         MCSS = await ethers.getContractFactory("MapCrossChainService");
       // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mcss = await MCSS.deploy();
        console.log("mcss address:",mcss.address);
        StandardToken = await ethers.getContractFactory("StandardToken");

        standardToken = await  StandardToken.deploy("MapToken","MP");

        await mcss.initialize(mcsData.weth,standardToken.address,mcsData.lightnode);

    });

    it('mcs set', async function () {

        await mcss.addAuthToken([standardToken.address]);

        await mcss.setBridge(mcsData.mcsRelay,212);

        await mcss.setCanBridgeToken(standardToken.address,212,"true");

        await mcss.setCanBridgeToken(standardToken.address,1313161555,"true");

        let mintRole = await  standardToken.MINTER_ROLE();

        await standardToken.grantRole(mintRole,mcss.address);

        await standardToken.mint(addr1.address,"100000000000000000000000000");

        expect(await standardToken.balanceOf(addr1.address)).to.equal("100000000000000000000000000");
    });

    it('transferOutToken test',async function () {
         address2Bytes = await mcss._addressToBytes(addr2.address);

        await standardToken.connect(addr1).approve(mcss.address,"10000000000000000000000000000000000")

        await mcss.connect(addr1).transferOutToken(standardToken.address,address2Bytes,"100000000000000000000000",212);

        expect(await standardToken.totalSupply()).to.equal(BigNumber.from("99900000000000000000000000"));

        await mcss.removeAuthToken([standardToken.address]);

        expect(await mcss.checkAuthToken(standardToken.address)).to.equal(false);

        await mcss.connect(addr1).transferOutToken(standardToken.address,address2Bytes,"900000000000000000000000",212);

        expect(await standardToken.totalSupply()).to.equal(BigNumber.from("99900000000000000000000000"));
        expect(await standardToken.connect(addr1).balanceOf(addr1.address)).to.equal("99000000000000000000000000")

    });

    it('transferIn test ', async function () {
        standardEthToken = await ethers.getContractAt("StandardToken",mcsData.standardToken);
        mapTokenEthToken = await ethers.getContractAt("StandardToken",mcsData.mapToken);

        expect(await standardEthToken.totalSupply()).to.equal("209900100000000000000000");

        await mcss.addAuthToken([mcsData.standardToken]);

        expect(await mcss.checkAuthToken(mcsData.standardToken)).to.equal(true);

        await mcss.transferIn(1313161555,mcsData.map2ethStandardToken);

        expect(await standardEthToken.totalSupply()).to.equal("209900300000000000000000");

        //balance 2000000000000000000
        console.log(await mapTokenEthToken.balanceOf(mcss.address));

        // mcs value mapToken 150000000000000000
        await mcss.transferIn(1313161555,mcsData.map2ethMapToken);

        expect(await mapTokenEthToken.balanceOf(mcss.address)).to.equal("1850000000000000000")

    });

    it('transferOutNative', async function () {
        wethEthToken = await ethers.getContractAt("StandardToken",mcsData.weth);

        await mcss.setCanBridgeToken("0x0000000000000000000000000000000000000000",1313161555,"true");

        await mcss.connect(owner).transferOutNative(address2Bytes,1313161555,{value:"100000000000000000"});

        expect(await wethEthToken.balanceOf(mcss.address)).to.equal("100000000000000000")

    });



})