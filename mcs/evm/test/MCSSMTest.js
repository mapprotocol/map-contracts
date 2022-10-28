const { ethers } = require("hardhat");
const { expect } = require("chai");
const mcsData = require('./mcsData');
require("solidity-coverage");
const { BigNumber} = require("ethers");

describe("MAPCrossChainService start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;
    let addr6;

    let MCSS;
    let mcss;

    let StandardToken;
    let standardToken;

    let UToken;
    let usdt;

    let Wrapped;
    let wrapped;

    let LightNode;
    let lightNode;


    let initData;

    let address2Bytes;

    beforeEach(async function () {

        [addr6,owner, addr1, addr2, addr3, addr4, addr5,...addrs] = await ethers.getSigners();

    });

    it("constract deploy init", async function () {
        console.log("addr6 address:",addr6.address);
        MCSS = await ethers.getContractFactory("MapCrossChainService");
        mcss = await MCSS.deploy();
        console.log("mcss address:",mcss.address);
        StandardToken = await ethers.getContractFactory("StandardToken");
        standardToken = await StandardToken.deploy("MapToken","MP");
        console.log("StandardToken:",standardToken.address);

        UToken = await ethers.getContractFactory("StandardToken");
        usdt = await  UToken.deploy("U Toeken","USDT");
        console.log("UToken:",usdt.address);

        Wrapped = await ethers.getContractFactory("Wrapped");
        wrapped = await Wrapped.deploy();
        console.log("Wrapped:",Wrapped.address);

        LightNode = await ethers.getContractFactory("LightNode");
        lightNode = await  LightNode.deploy();

        let data  = await mcss.initialize(wrapped.address,lightNode.address);

        initData = data.data;

    });

    it('UUPS test', async function () {
        const MapCrossChainServiceProxy = await ethers.getContractFactory("MapCrossChainServiceProxy");
        let mcssp = await MapCrossChainServiceProxy.deploy(mcss.address,initData);
        await mcssp.deployed()
        mcss = MCSS.connect(addr6).attach(mcssp.address);
    });

    it('mcs set', async function () {

        await mcss.addAuthToken([standardToken.address]);

        await mcss.setMcsRelay(212,mcsData.mcsRelay);

        await mcss.setCanBridgeToken(standardToken.address,212,"true");

        await mcss.setCanBridgeToken(standardToken.address,1313161555,"true");

        let mintRole = await standardToken.MINTER_ROLE();

        await standardToken.grantRole(mintRole,mcss.address);

        await standardToken.mint(addr1.address,"100000000000000000000000000");

        expect(await standardToken.balanceOf(addr1.address)).to.equal("100000000000000000000000000");
    });

    it('transferOutToken test',async function () {
        //console.log(addr2.address);
        //address2Bytes = await mcss._addressToBytes(addr2.address);
        address2Bytes = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";

        await standardToken.connect(addr1).approve(mcss.address,"10000000000000000000000000000000000")

        await mcss.connect(addr1).transferOutToken(standardToken.address,address2Bytes,"100000000000000000000000",212);

        expect(await standardToken.totalSupply()).to.equal(BigNumber.from("99900000000000000000000000"));

        await mcss.removeAuthToken([standardToken.address]);

        expect(await mcss.checkAuthToken(standardToken.address)).to.equal(false);

        await mcss.connect(addr1).transferOutToken(standardToken.address,address2Bytes,"900000000000000000000000",212);

        expect(await standardToken.totalSupply()).to.equal(BigNumber.from("99900000000000000000000000"));
        expect(await standardToken.connect(addr1).balanceOf(addr1.address)).to.equal("99000000000000000000000000")

    });

    it('map transferIn test ', async function () {
        await mcss.addAuthToken([standardToken.address]);

        await mcss.transferIn(212,mcsData.map2ethStandardToken);

        expect(await standardToken.totalSupply()).to.equal("99900000100000000000000000");

        expect(await usdt.balanceOf(mcss.address)).to.equal("0");

        await mcss.transferIn(212,mcsData.map2ethMapToken0);

        expect(await standardToken.totalSupply()).to.equal("99900000100000000000000000");
        expect(await usdt.balanceOf(mcss.address)).to.equal("0");

        await wrapped.deposit({value:"300000000000000000"});
        await wrapped.transfer(mcss.address,"300000000000000000");

        await mcss.transferIn(212,mcsData.map2ethNative);

        expect(await wrapped.balanceOf(mcss.address)).to.equal("0");

        await usdt.mint(mcss.address,"5000000000000000000")

        await mcss.transferIn(212,mcsData.map2ethMapToken);
        expect(await usdt.balanceOf(mcss.address)).to.equal("0");
        expect(await usdt.totalSupply()).to.equal("5000000000000000000");

    });

    it('depositOut test', async function () {
        await mcss.connect(addr1).depositOutToken(
            standardToken.address,
            addr1.address,
            addr3.address,
            "2000000000000000000000"
        )
        expect(await standardToken.balanceOf(addr1.address)).to.equal("98998000000000000000000000");

        console.log( BigNumber.from(await ethers.provider.getBalance(addr2.address)));
        await mcss.connect(addr2).depositOutNative(
            addr2.address,
            addr4.address,
            {
                value:"1000000000000000000"
            }
        )

        // expect(await ethers.provider.getBalance(mcss.address)).to.equal("9998999928426602550800");
        expect(await wrapped.balanceOf(mcss.address)).to.equal("1000000000000000000")
    });

    it('near transferIn test ', async function () {

        //250000000000000000
        await usdt.mint(mcss.address,"250000000000000000")
        await mcss.transferIn(212,mcsData.near2ethW);
        expect(await usdt.totalSupply()).to.equal("5250000000000000000");
        expect(await usdt.balanceOf(mcss.address)).to.equal("0");
        // console.log(await standardToken.balanceOf("0x2e784874ddb32cd7975d68565b509412a5b519f4"));

        expect(await standardToken.totalSupply()).to.equal("99900000100000000000000000");

        await mcss.transferIn(212,mcsData.near2eth001);

        expect(await standardToken.totalSupply()).to.equal("99900000350000000000000000");

        await mcss.transferIn(212,mcsData.near2et000);

        expect(await wrapped.balanceOf(mcss.address)).to.equal("750000000000000000");

    });


    it('transferOutNative', async function () {
        await mcss.setCanBridgeToken("0x0000000000000000000000000000000000000000",1313161555,"true");

        await mcss.connect(owner).transferOutNative(address2Bytes,1313161555,{value:"100000000000000000"});

        expect(await wrapped.balanceOf(mcss.address)).to.equal("850000000000000000")

    });

    it('withdraw test', async function () {
        console.log(await ethers.provider.getBalance(mcss.address));

        await mcss.withdraw(
            "0x0000000000000000000000000000000000000000",
            addr5.address,
            "850000000000000000"
        )
        expect(await ethers.provider.getBalance(mcss.address)).to.equal("0");
        expect(await ethers.provider.getBalance(addr5.address)).to.equal("10000850000000000000000");

        await mcss.withdraw(
            standardToken.address,
            addr5.address,
            "2000000000000000000000"
        )
        expect(await standardToken.balanceOf(mcss.address)).to.equal("900000000000000000000000");

    });

    it('set test', async function () {
        await mcss.setPause();
        expect(await mcss.paused()).to.equal(true);
        await mcss.setUnpause();
        expect(await mcss.paused()).to.equal(false);

        await expect(mcss.connect(addr3).setPause()).to.be.revertedWith("lightnode :: only admin")
    });

    it('admin test', async function () {

        await expect(mcss.changeAdmin("0x0000000000000000000000000000000000000000")).to.be.revertedWith("zero address")

        await mcss.changeAdmin(addr5.address);

        expect(await mcss.getAdmin()).to.equal(addr5.address);

    });

    it('Upgrade', async function () {
        let MCSSUpGrade = await ethers.getContractFactory("MapCrossChainService");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        let mcssUpGrade = await MCSSUpGrade.deploy();
        await mcssUpGrade.deployed();

        mcss.connect(addr5).upgradeTo(mcssUpGrade.address);

        expect(await mcss.getImplementation()).to.equal(mcssUpGrade.address);

        await expect(mcss.transferIn(212,mcsData.near2et000)).to.be.revertedWith("order exist");

    });
})


