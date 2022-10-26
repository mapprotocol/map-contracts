const { ethers } = require("hardhat");
const { expect } = require("chai");
const mcsRelayData = require('./mcsRelayData');
require("solidity-coverage");


describe("MAPCrossChainServiceRelay start test", function () {

    let owner;
    let addr1;
    let addr2;
    let addr3;
    let addr4;
    let addr5;
    let addr6;
    let addrs;

    let MCSSRelay;
    let mcssR;

    let MapVault;
    let mapVault;

    let MapVaultU;
    let mapVaultU;

    let StandardToken;
    let standardToken;

    let UToken;
    let usdt;

    let Wrapped;
    let wrapped;

    let TokenRegister;
    let tokenRegister;

    let FeeCenter;
    let feeCenter;

    let LightClientManager;
    let lightClientManager;

    let address2Bytes;
    let initData;

    beforeEach(async function () {

        [deployer,owner, addr1, addr2, addr3, addr4, addr5,addr6,...addrs] = await ethers.getSigners();

    });

    it("mcsRelay contract deploy init", async function () {
        console.log("deployer address:",deployer.address)

        MCSSRelay = await ethers.getContractFactory("MAPCrossChainServiceRelay");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mcssR = await MCSSRelay.deploy();
        console.log("mcssR address:",mcssR.address);

        StandardToken = await ethers.getContractFactory("StandardToken");
        standardToken = await  StandardToken.deploy("MapToken","MP");

        UToken = await ethers.getContractFactory("StandardToken");
        usdt = await  UToken.deploy("U Toeken","USDT");

        Wrapped = await ethers.getContractFactory("Wrapped");
        wrapped = await Wrapped.deploy();

        TokenRegister = await ethers.getContractFactory("TokenRegister");
        tokenRegister = await TokenRegister.deploy();
        console.log("TokenRegister address",tokenRegister.address);

        FeeCenter = await ethers.getContractFactory("FeeCenter");
        feeCenter = await FeeCenter.deploy();
        console.log("FeeCenter address",feeCenter.address);

        LightClientManager = await ethers.getContractFactory("LightClientManager");
        lightClientManager = await LightClientManager.deploy();
        console.log("LightClientManager   address:",lightClientManager.address);

        MapVault = await ethers.getContractFactory("MAPVaultToken");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mapVault = await MapVault.deploy();
        console.log("MapVault  address:",mapVault.address);
        await mapVault.initialize(standardToken.address,"MapVaultToken","MVT","18");

        MapVaultU = await ethers.getContractFactory("MAPVaultToken");
        mapVaultU = await MapVaultU.deploy();

        await mapVaultU.initialize(usdt.address,"MapVaultTokenUsdt","UVT","18");

        let data = await mcssR.initialize(wrapped.address,lightClientManager.address);
        initData = data.data;
    });

    it('UUPS test', async function () {
        const MAPCrossChainServiceRelayProxy = await ethers.getContractFactory("MAPCrossChainServiceRelayProxy");
        let mcssRP = await MAPCrossChainServiceRelayProxy.deploy(mcssR.address,initData);
        await mcssRP.deployed()

        mcssR = MCSSRelay.attach(mcssRP.address);

    });

    it('mcsRelay contract set ', async function () {
        await mcssR.setTokenRegister(tokenRegister.address);

        expect(await mcssR.tokenRegister()).to.equal(tokenRegister.address);

        await mcssR.setMcsContract(34434,mcsRelayData.mcsETH);

        await mcssR.setMcsContract(1313161555,mcsRelayData.mcsNear);

        await mcssR.setChain("near",1313161555);

        await mcssR.setFeeCenter(feeCenter.address);
        //await mcssR.setFeeCenter(mcsRelayData.feeCenter);
        expect(await mcssR.feeCenter()).to.equal(feeCenter.address);

        await tokenRegister.addAuthToken([standardToken.address]);

        await mapVault.addManager(mcssR.address);
        await mapVaultU.addManager(mcssR.address);

        await feeCenter.setChainTokenGasFee(34434,usdt.address,"1000000000000000","2000000000000000000","5000")

        await feeCenter.setDistributeRate(0,addr2.address,"4000")
        await feeCenter.setDistributeRate(1,addr3.address,"2000")
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

        await mcssR.setVaultBalance(34434,wrapped.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(212,wrapped.address,"1000000000000000000000000000000");
        await mcssR.setVaultBalance(1313161555,wrapped.address,"1000000000000000000000000000000");

        await mcssR.setVaultBalance(34434,"0x0000000000000000000000000000000000000000","1000000000000000000000000000000");
        await mcssR.setVaultBalance(212,"0x0000000000000000000000000000000000000000","1000000000000000000000000000000");
        await mcssR.setVaultBalance(1313161555,"0x0000000000000000000000000000000000000000","1000000000000000000000000000000");

    });

    it('mcsRelay set token decimals', async function () {
        await tokenRegister.setTokenOtherChainDecimals(standardToken.address,212,18);
        await tokenRegister.setTokenOtherChainDecimals(standardToken.address,34434,18);
        await tokenRegister.setTokenOtherChainDecimals(standardToken.address,1313161555,24);

        await tokenRegister.setTokenOtherChainDecimals(wrapped.address,212,18);
        await tokenRegister.setTokenOtherChainDecimals(wrapped.address,34434,18);
        await tokenRegister.setTokenOtherChainDecimals(wrapped.address,1313161555,24);

        await tokenRegister.setTokenOtherChainDecimals(usdt.address,212,18);
        await tokenRegister.setTokenOtherChainDecimals(usdt.address,34434,18);
        await tokenRegister.setTokenOtherChainDecimals(usdt.address,1313161555,24);

        await tokenRegister.setTokenOtherChainDecimals("0x0000000000000000000000000000000000000000",212,18);
        await tokenRegister.setTokenOtherChainDecimals("0x0000000000000000000000000000000000000000",34434,18);
        await tokenRegister.setTokenOtherChainDecimals("0x0000000000000000000000000000000000000000",1313161555,24);

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

        await tokenRegister.removeAuthToken([standardToken.address]);

        await mcssR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",1313161555)

        expect(await standardToken.totalSupply()).to.equal("1000000000000000000");

        expect(await standardToken.balanceOf(owner.address)).to.equal("0");
    });

    it('transferOutNative test ', async function () {

        await mcssR.connect(owner).transferOutNative(address2Bytes,1313161555,{value:"100000000000000000"});

        expect(await wrapped.balanceOf(mcssR.address)).to.equal("100000000000000000")
    });


    it('transferIn test ', async function () {

        await tokenRegister.addAuthToken([standardToken.address]);
        //console.log(await tokenRegister.getTargetToken(1313161555,212))

        console.log(await usdt.balanceOf(mcssR.address));
        console.log(await wrapped.balanceOf(mcssR.address));
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

        expect(await wrapped.balanceOf(mcssR.address)).to.equal("100000000000000000");
        await wrapped.deposit({value:"50000000000000000"});
        await wrapped.transfer(mcssR.address,"50000000000000000");
        await mcssR.transferIn(1313161555,mcsRelayData.near2map000);
        expect(await wrapped.balanceOf(mcssR.address)).to.equal("0");

    });

    it('eth2map transferIn test', async function () {
        expect(await usdt.balanceOf(mcssR.address)).to.equal("0");
        await usdt.mint(mcssR.address,"100000000000000000000");
        await mcssR.transferIn(34434,mcsRelayData.eth2mapMapToken);
        expect(await usdt.balanceOf(mcssR.address)).to.equal("0")

        await mcssR.transferIn(34434,mcsRelayData.eth2mapStandardToken);
        expect(await standardToken.totalSupply()).to.equal("301150000000000000000");

        expect(await wrapped.balanceOf(mcssR.address)).to.equal("0");
        await wrapped.deposit({value:"2000000000000000000"});
        await wrapped.transfer(mcssR.address,"2000000000000000000");
        await mcssR.transferIn(34434,mcsRelayData.eth2mapNative);
        expect(await wrapped.balanceOf(mcssR.address)).to.equal("0");
    });

    it('depositIn test ', async function () {
        // expect(await usdt.balanceOf(mcssR.address)).to.equal("0")
        await feeCenter.setTokenVault(usdt.address,mapVaultU.address)
        expect(await mapVaultU.totalSupply()).to.equal("0");
        await usdt.mint(mcssR.address,"150000000000000000");
        console.log(await usdt.balanceOf(mcssR.address));
        await mcssR.depositIn(1313161555,mcsRelayData.near2mapDeposite);
        expect(await usdt.balanceOf(mcssR.address)).to.equal("0")
        expect(await mapVaultU.balanceOf("0x2e784874ddb32cd7975d68565b509412a5b519f4")).to.equal("150000000000000000")
        expect(await mapVaultU.totalSupply()).to.equal("150000000000000000");

        await mcssR.setMcsContract(34434,"0xAC25DeA31A410900238c8669eD9973f328919160");

        await feeCenter.setTokenVault(standardToken.address,mapVault.address)

        await mcssR.depositIn(34434,mcsRelayData.eth2mapDeposite);

        expect(await standardToken.totalSupply()).to.equal("10301150000000000000000");
        expect(await mapVault.balanceOf("0x2e784874ddb32cd7975d68565b509412a5b519f4")).to.equal("10000000000000000000000")
        expect(await mapVault.totalSupply()).to.equal("10000000000000000000000");
    });

    it('withdraw test', async function () {
        console.log(await ethers.provider.getBalance(mcssR.address));

        await wrapped.connect(addr4).deposit({value:"1000000000000000000"});
        await wrapped.connect(addr4).transfer(mcssR.address,"1000000000000000000");

        await mcssR.withdraw(
            "0x0000000000000000000000000000000000000000",
            addr6.address,
            "1000000000000000000"
        )
        expect(await wrapped.balanceOf(mcssR.address)).to.equal("0");
        expect(await ethers.provider.getBalance(addr6.address)).to.equal("10001000000000000000000");

        //await standardToken.mint(mcssR.address,"1000000000000000000000")
        await mcssR.withdraw(
            standardToken.address,
            addr5.address,
            "1000000000000000000"
        )
        expect(await standardToken.balanceOf(mcssR.address)).to.equal("0");

    });

    it('error test', async function () {

        //assert.equal(await mcssR.transferIn(888,mcsRelayData.near2eth000),"fail");
        await expect(mcssR.transferIn(888,mcsRelayData.near2eth000)).to.be.revertedWith("fail")

    });

    it('set test', async function () {
        console.log(await mcssR.getAdmin());
        await mcssR.setPause();
        expect(await mcssR.paused()).to.equal(true);
        await mcssR.setUnpause();
        expect(await mcssR.paused()).to.equal(false);

        await expect(mcssR.connect(addr3).setPause()).to.be.revertedWith("mcsRelay :: only admin")

    });

    it('admin test', async function () {

        await expect(mcssR.changeAdmin("0x0000000000000000000000000000000000000000")).to.be.revertedWith("zero address")

        await mcssR.changeAdmin(addr5.address);

        expect(await mcssR.getAdmin()).to.equal(addr5.address);

    });

    it('Upgrade', async function () {
        let MCSSRelayUpGrade = await ethers.getContractFactory("MAPCrossChainServiceRelay");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        let mcssRUpGrade = await MCSSRelayUpGrade.deploy();
        await mcssRUpGrade.deployed();

        mcssR.connect(addr5).upgradeTo(mcssRUpGrade.address);

        expect(await mcssR.getImplementation()).to.equal(mcssRUpGrade.address);

        await expect(mcssR.transferIn(1313161555,mcsRelayData.near2mapW)).to.be.revertedWith("order exist");
    });

    it('collectChainFee test', async function () {
        await usdt.mint(owner.address,"1000000000000000000");
        await usdt.connect(owner).approve(mcssR.address,"100000000000000000000");
        await mcssR.connect(owner).transferOutToken(usdt.address,address2Bytes,"1000000000000000000",34434);

        expect(await usdt.balanceOf(mcssR.address)).to.be.equal("500000000000000000");
        expect(await usdt.balanceOf(mapVaultU.address)).to.be.equal("152000000000000000");
        expect(await usdt.balanceOf(addr3.address)).to.be.equal("1000000000000000");
        await mcssR.connect(addr5).setLightClientManager(addr4.address);
        expect(await mcssR.lightClientManager()).to.be.equal(addr4.address);

    });
})
