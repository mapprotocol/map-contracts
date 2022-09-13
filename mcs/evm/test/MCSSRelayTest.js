const { ethers } = require("hardhat");
const { expect } = require("chai");
const mcsRelayData = require('./mcsRelayData');
require("solidity-coverage");
//const { BigNumber, BytesLike, Contract, ContractTransaction } = require("ethers");
const {tokenregister} = require("./mcsRelayData");
const mcsData = require("./mcsData");
const {lightnode} = require("./mcsData");

describe("MAPCrossChainServiceRelay start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;
    let addrs;

    let MCSSRelay;
    let mcssR;

    let MapVault;
    let mapVault;

    let StandardToken;
    let standardToken;

    let UToken;
    let usdt;

    let WETH;
    let weth;

    let TokenRegister;
    let tokenRegister;

    let FeeCenter;
    let feeCenter;

    let LightClientManager;
    let lightClientManager

    let address2Bytes

    beforeEach(async function () {

        [deployer,owner, addr1, addr2, addr3, addr4, addr5,...addrs] = await ethers.getSigners();

    });

    it("mcsRelay contract deploy init", async function () {
        console.log("deployer address:",deployer.address)
        MCSSRelay = await ethers.getContractFactory("MAPCrossChainServiceRelay");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mcssR = await MCSSRelay.deploy();
        console.log("mcssR address:",mcssR.address);

        MapVault = await ethers.getContractFactory("MAPVaultToken");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mapVault = await MapVault.deploy();
        console.log("MapVault  address:",mapVault.address);

        await mapVault.initialize(mcsRelayData.standardToken,"MapVaultToken","MVT","18");
        StandardToken = await ethers.getContractFactory("StandardToken");
        standardToken = await  StandardToken.deploy("MapToken","MP");

        UToken = await ethers.getContractFactory("StandardToken");
        usdt = await  UToken.deploy("U Toeken","USDT");

        WETH = await ethers.getContractFactory("WETH9");
        weth = await  WETH.deploy();

        TokenRegister = await ethers.getContractFactory("TokenRegister");
        tokenRegister = await TokenRegister.deploy();
        console.log("TokenRegister address",tokenRegister.address);

        FeeCenter = await ethers.getContractFactory("FeeCenter");
        feeCenter = await FeeCenter.deploy();
        console.log("FeeCenter address",feeCenter.address);

        LightClientManager = await ethers.getContractFactory("LightClientManager");
        lightClientManager = await LightClientManager.deploy();
        console.log("LightClientManager   address:",lightClientManager.address);

        await mcssR.initialize(weth.address,standardToken.address,lightClientManager.address);
        // await mcssR.initialize(mcsRelayData.weth,mcsRelayData.standardToken,mcsRelayData.lightClientManager);

    });

    it('mcsRelay contract set ', async function () {
        await mcssR.setTokenRegister(tokenRegister.address);
        //await mcssR.setTokenRegister(mcsRelayData.tokenregister);
        expect(await mcssR.tokenRegister()).to.equal(tokenRegister.address);

        await mcssR.setBridageAddress(34434,mcsRelayData.mcsETH);
        await mcssR.setBridageAddress(1313161555,mcsRelayData.mcsNear);

        await mcssR.setIdTable(1313161555,1);

        await mcssR.setFeeCenter(feeCenter.address);
        //await mcssR.setFeeCenter(mcsRelayData.feeCenter);
        expect(await mcssR.feeCenter()).to.equal(feeCenter.address);

        await mcssR.addAuthToken([standardToken.address]);

        feeCenter.setTokenVault(usdt.address,mapVault.address)

        feeCenter.setChainTokenGasFee(34434,usdt.address,"1000000000000000","2000000000000000000","5000")

        feeCenter.setDistributeRate(0,addr2.address,"4000")
        feeCenter.setDistributeRate(1,addr3.address,"2000")
        //expect(await mcssR.checkAuthToken(standardToken.address)).to.equal("true");
    });

    it('TokenRegister set', async function () {
        await tokenRegister.regToken(34434,mcsRelayData.ethUsdtToken,usdt.address);
        await tokenRegister.regToken(34434,mcsRelayData.ethStanardToken,standardToken.address);
        await tokenRegister.regToken(212,usdt.address,mcsRelayData.ethUsdtToken);
        await tokenRegister.regToken(212,standardToken.address,mcsRelayData.ethStanardToken);
        await tokenRegister.regToken(1313161555,mcsRelayData.nearUsdtToken,usdt.address);
        await tokenRegister.regToken(1313161555,mcsRelayData.nearWethToken,standardToken.address);
        await tokenRegister.regToken(1313161555,"0x0000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000");
        await tokenRegister.regToken(212,"0x0000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000");
        await tokenRegister.regToken(34434,"0x0000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000");
    });

    it('mcsRelay setVaultBalance', async function () {

        await mcssR.setVaultBalance(34434,standardToken.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(212,standardToken.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(1313161555,standardToken.address,"1000000000000000000000000000000");

        await mcssR.setVaultBalance(34434,usdt.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(212,usdt.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(1313161555,usdt.address,"1000000000000000000000000000000");

        await mcssR.setVaultBalance(34434,weth.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(212,weth.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(1313161555,weth.address,"1000000000000000000000000000000");

        await mcssR.setVaultBalance(34434,"0x0000000000000000000000000000000000000000","1000000000000000000000000000000");
        await mcssR.setVaultBalance(212,"0x0000000000000000000000000000000000000000","1000000000000000000000000000000");
        await mcssR.setVaultBalance(1313161555,"0x0000000000000000000000000000000000000000","1000000000000000000000000000000");

    });

    it('mcsRelay set token decimals', async function () {
        await mcssR.setTokenOtherChainDecimals(standardToken.address,212,18);
        await mcssR.setTokenOtherChainDecimals(standardToken.address,34434,18);
        await mcssR.setTokenOtherChainDecimals(standardToken.address,1313161555,24);

        await mcssR.setTokenOtherChainDecimals(weth.address,212,18);
        await mcssR.setTokenOtherChainDecimals(weth.address,34434,18);
        await mcssR.setTokenOtherChainDecimals(weth.address,1313161555,24);

        await mcssR.setTokenOtherChainDecimals(usdt.address,212,18);
        await mcssR.setTokenOtherChainDecimals(usdt.address,34434,18);
        await mcssR.setTokenOtherChainDecimals(usdt.address,1313161555,24);

        await mcssR.setTokenOtherChainDecimals("0x0000000000000000000000000000000000000000",212,18);
        await mcssR.setTokenOtherChainDecimals("0x0000000000000000000000000000000000000000",34434,18);
        await mcssR.setTokenOtherChainDecimals("0x0000000000000000000000000000000000000000",1313161555,24);

    });

    it('transferOutToken', async function () {
        //chainID 31337
        //address2Bytes = await mcssR._addressToBytes(addr2.address);
        address2Bytes = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";

        let mintRole = await  standardToken.MINTER_ROLE();

        await standardToken.grantRole(mintRole,mcssR.address);

        await standardToken.mint(owner.address,"1000000000000000000");

        await standardToken.connect(owner).approve(mcssR.address,"100000000000000000000");

        await mcssR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",34434)

        expect(await standardToken.totalSupply()).to.equal("0");
        console.log(await standardToken.totalSupply());

        await standardToken.mint(owner.address,"1000000000000000000");

        await mcssR.removeAuthToken([standardToken.address]);

        await mcssR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",1313161555)

        expect(await standardToken.totalSupply()).to.equal("1000000000000000000");

        expect(await standardToken.balanceOf(owner.address)).to.equal("0");
    });

    it('transferOutNative test ', async function () {

        await mcssR.connect(owner).transferOutNative(address2Bytes,1313161555,{value:"100000000000000000"});

        expect(await weth.balanceOf(mcssR.address)).to.equal("100000000000000000")
    });


    it('transferIn test ', async function () {

        await mcssR.addAuthToken([standardToken.address]);
        //console.log(await tokenRegister.getTargetToken(1313161555,212))

        console.log(await usdt.balanceOf(mcssR.address));
        console.log(await weth.balanceOf(mcssR.address));
        console.log(await standardToken.balanceOf(mcssR.address));

        await mcssR.transferIn(1313161555,mcsRelayData.near2eth001);
        await mcssR.transferIn(1313161555,mcsRelayData.near2ethW);
        await mcssR.transferIn(1313161555,mcsRelayData.near2eth000);

        expect(await usdt.balanceOf(mcssR.address)).to.equal("0");
        await usdt.mint(mcssR.address,"150000000000000000");
        await mcssR.transferIn(1313161555,mcsRelayData.near2map001);
        expect(await usdt.balanceOf(mcssR.address)).to.equal("0")

        await mcssR.transferIn(1313161555,mcsRelayData.near2mapW);
        expect(await standardToken.totalSupply()).to.equal("1150000000000000000");

        expect(await weth.balanceOf(mcssR.address)).to.equal("100000000000000000");
        await weth.deposit({value:"50000000000000000"});
        await weth.transfer(mcssR.address,"50000000000000000");
        await mcssR.transferIn(1313161555,mcsRelayData.near2map000);
        expect(await weth.balanceOf(mcssR.address)).to.equal("0");

    });

    it('eth2map transferIn test', async function () {
        expect(await usdt.balanceOf(mcssR.address)).to.equal("0");
        await usdt.mint(mcssR.address,"100000000000000000000");
        await mcssR.transferIn(34434,mcsRelayData.eth2mapMapToken);
        expect(await usdt.balanceOf(mcssR.address)).to.equal("0")

        await mcssR.transferIn(34434,mcsRelayData.eth2mapStandardToken);
        expect(await standardToken.totalSupply()).to.equal("301150000000000000000");

        expect(await weth.balanceOf(mcssR.address)).to.equal("0");
        await weth.deposit({value:"2000000000000000000"});
        await weth.transfer(mcssR.address,"2000000000000000000");
        await mcssR.transferIn(34434,mcsRelayData.eth2mapNative);
        expect(await weth.balanceOf(mcssR.address)).to.equal("0");
    });

})