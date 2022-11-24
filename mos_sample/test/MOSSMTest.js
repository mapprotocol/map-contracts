const { ethers } = require("hardhat");
const { expect } = require("chai");
const mosData = require('./mosData');
require("solidity-coverage");
const { BigNumber} = require("ethers");

describe("MAPOmnichainServiceV2 start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;
    let addr6;
    let addr7;
    let addr8;
    let addr9;

    let MOSS;
    let moss;

    let StandardToken;
    let standardToken;

    let UToken;
    let usdt;

    let LightNode;
    let lightNode;


    let address2Bytes;

    let receiver = "0x2E784874ddB32cD7975D68565b509412A5B519F4";

    beforeEach(async function () {

        [addr6,owner, addr1, addr2, addr3, addr4, addr5,addr7,addr8,addr9,...addrs] = await ethers.getSigners();

    });

    it("constract deploy init", async function () {

        let MOSS1 = await ethers.getContractFactory("MAPOmnichainServiceV2");
        let moss1 = await MOSS1.deploy(addr1.address);
        console.log("moss address:",moss1.address);

        StandardToken = await ethers.getContractFactory("MintableToken");
        standardToken = await StandardToken.deploy("MapToken","MP");
        console.log("StandardToken:",standardToken.address);

        UToken = await ethers.getContractFactory("MintableToken");
        usdt = await  UToken.deploy("U Toeken","USDT");
        console.log("UToken:",usdt.address);


        LightNode = await ethers.getContractFactory("LightNode");
        lightNode = await  LightNode.deploy();

        MOSS = await ethers.getContractFactory("MAPOmnichainServiceV2");
        moss = await MOSS.deploy(lightNode.address);
        console.log("moss address:",moss.address);

    });

     it('mos set', async function () {

         await standardToken.mint(addr1.address,"100000000000000000000000000");

     });

    it('transferOutToken test',async function () {

        address2Bytes = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";

        await standardToken.connect(addr1).approve(moss.address,"10000000000000000000000000000000000")

        await moss.connect(addr1).transferOutToken(standardToken.address,address2Bytes,"10000000000000000000000000",34434);
  
        expect(await standardToken.totalSupply()).to.equal(BigNumber.from("100000000000000000000000000"));
     
        expect(await standardToken.connect(addr1).balanceOf(addr1.address)).to.equal("90000000000000000000000000")

        expect(await standardToken.balanceOf(moss.address)).to.equal("10000000000000000000000000");
    });

    it('map transferIn test ', async function () {

        await moss.transferIn(212,mosData.map2ethStandardToken);

        expect(await standardToken.balanceOf(moss.address)).to.equal("9999999900000000000000000");

        expect(await usdt.balanceOf(moss.address)).to.equal("0");

        expect(await usdt.balanceOf(moss.address)).to.equal("0");

        await moss.transferIn(212,mosData.map2ethMapToken0);

        expect(await standardToken.balanceOf(moss.address)).to.equal("9999999900000000000000000");

        expect(await standardToken.totalSupply()).to.equal("100000000000000000000000000");
        expect(await usdt.balanceOf(moss.address)).to.equal("0");

        await usdt.mint(moss.address,"5000000000000000000");

        await moss.transferIn(212,mosData.map2ethMapToken);
        expect(await usdt.balanceOf(moss.address)).to.equal(0);
        expect(await usdt.balanceOf(receiver)).to.equal("5000000000000000000");

    });

})


