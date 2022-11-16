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

    let EvmDecoder;
    let evmDecoder;

    let NearDecoder;
    let nearDecoder;

    let MOSSRelay;
    let mossR;

    let MapVault;
    let mapVault;

    let MapVaultU;
    let mapVaultU;

    let MapVaultW;
    let mapVaultW;

    let StandardToken;
    let standardToken;

    let UToken;
    let usdt;

    let Wrapped;
    let wrapped;

    let TokenRegister;
    let tokenRegister;
    

    let LightClientManager;
    let lightClientManager;

    let address2Bytes;
    let initData;

    beforeEach(async function () {

        [deployer,owner, addr1, addr2, addr3, addr4, addr5,addr6,addr7,addr8] = await ethers.getSigners();

    });

    it("MAPOmnichainServiceRelayV2 contract deploy init", async function () {
        console.log("deployer address:",deployer.address)


        EvmDecoder = await ethers.getContractFactory("EvmDecoder");
        evmDecoder = await EvmDecoder.deploy();
        console.log("EvmDecoder address:",evmDecoder.address);

        NearDecoder = await ethers.getContractFactory("NearDecoder");
        nearDecoder = await NearDecoder.deploy();
        console.log("NearDecoder address:",nearDecoder.address);

        MOSSRelay = await ethers.getContractFactory("MAPOmnichainServiceRelayV2");
        // moss = await ethers.getContractAt("MapCrossChainService",mosData.mos);
        mossR = await MOSSRelay.deploy();
        console.log("mossR address:",mossR.address);

        StandardToken = await ethers.getContractFactory("MintableToken");
        standardToken = await  StandardToken.deploy("MapToken","MP");

        UToken = await ethers.getContractFactory("MintableToken");
        usdt = await  UToken.deploy("U Toeken","USDT");

        Wrapped = await ethers.getContractFactory("Wrapped");
        wrapped = await Wrapped.deploy();
        console.log("Wrapped:",wrapped.address)

        TokenRegister = await ethers.getContractFactory("TokenRegisterV2");
        tokenRegister = await TokenRegister.deploy();
        console.log("TokenRegister address",tokenRegister.address);


        LightClientManager = await ethers.getContractFactory("LightClientManager");
        lightClientManager = await LightClientManager.deploy();
        console.log("LightClientManager   address:",lightClientManager.address);

        MapVault = await ethers.getContractFactory("VaultTokenV2");
        // moss = await ethers.getContractAt("MapCrossChainService",mosData.mos);
        mapVault = await MapVault.deploy(standardToken.address,"MapVaultToken","MVT");
        console.log("MapVault  address:",mapVault.address);
        //await mapVault.initialize(standardToken.address,"MapVaultToken","MVT","18");

        MapVaultU = await ethers.getContractFactory("VaultTokenV2");
        mapVaultU = await MapVaultU.deploy(usdt.address,"MapVaultTokenUsdt","UVT");

        MapVaultW = await ethers.getContractFactory("VaultTokenV2");
        mapVaultW = await MapVaultU.deploy(wrapped.address,"MapVaultTokenWrapped","WVT");

        //await mapVaultU.initialize(usdt.address,"MapVaultTokenUsdt","UVT","18");

        let data = await mossR.initialize(wrapped.address,lightClientManager.address);
        initData = data.data;
    });

    it('UUPS test', async function () {
        const MAPCrossChainServiceRelayProxy = await ethers.getContractFactory("MAPOmnichainServiceProxyV2");
        let mossRP = await MAPCrossChainServiceRelayProxy.deploy(mossR.address,initData);
        await mossRP.deployed()

        mossR = MOSSRelay.attach(mossRP.address);

    });

    it('mosRelay contract set ', async function () {
        await mossR.setTokenManager(tokenRegister.address);

        expect(await mossR.tokenRegister()).to.equal(tokenRegister.address);

        await mossR.registerChain(34434,mosRelayData.mosETH,1);

        await mossR.registerChain(1313161555,mosRelayData.mosNear,2);
        await mossR.registerChain("5566818579631833089",mosRelayData.mosNear,2);

        expect(await mossR.chainTypes(34434)).to.equal(1)

        await mapVault.addManager(mossR.address);
        await mapVaultU.addManager(mossR.address);
        await mapVaultW.addManager(mossR.address);

        await mossR.setDistributeRate(0,addr2.address,"400000")
        await mossR.setDistributeRate(1,addr3.address,"200000")
        //expect(await mossR.checkAuthToken(standardToken.address)).to.equal("true");
    });

    it('TokenRegister set', async function () {
        await tokenRegister.registerToken(usdt.address,mapVaultU.address,false);
        await tokenRegister.registerToken(standardToken.address,mapVault.address,true);
        await tokenRegister.registerToken(wrapped.address,mapVaultW.address,false);

        await tokenRegister.mapToken(usdt.address,34434,mosRelayData.ethUsdtToken,18);
        await tokenRegister.mapToken(usdt.address,97,mosRelayData.ethUsdtToken,18);
        await tokenRegister.mapToken(standardToken.address,34434,mosRelayData.ethStanardToken,18);
        await tokenRegister.mapToken(standardToken.address,97,mosRelayData.ethStanardToken,18);
        await tokenRegister.mapToken(usdt.address,212,usdt.address,18);
        await tokenRegister.mapToken(standardToken.address,212,standardToken.address,18);
        await tokenRegister.mapToken(usdt.address,1313161555,mosRelayData.nearUsdtToken,24);
        await tokenRegister.mapToken(usdt.address,"5566818579631833089",mosRelayData.nearUsdtToken,24);
        await tokenRegister.mapToken(standardToken.address,1313161555,mosRelayData.nearWethToken,24);
        await tokenRegister.mapToken(standardToken.address,"5566818579631833089",mosRelayData.nearWethToken,24);
        await tokenRegister.mapToken(wrapped.address,1313161555,"0x0000000000000000000000000000000000000000",24);
        await tokenRegister.mapToken(wrapped.address,"5566818579631833089","0x0000000000000000000000000000000000000000",24);
        await tokenRegister.mapToken(wrapped.address,212,wrapped.address,18);
        await tokenRegister.mapToken(wrapped.address,34434,"0x0000000000000000000000000000000000000000",18);
        await tokenRegister.mapToken(wrapped.address,97,"0xae13d989dac2f0debff460ac112a837c89baa7cd",18);

        await tokenRegister.setTokenFee(usdt.address,34434,"1000000000000000","2000000000000000000","500000")

    });


    it('transferOutToken', async function () {
        //chainID 31337
        //address2Bytes = await mossR._addressToBytes(addr2.address);
        address2Bytes = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";

        let mintRole = await  standardToken.MINTER_ROLE();

        await standardToken.grantRole(mintRole,mossR.address);

        await standardToken.mint(owner.address,"1000000000000000000");

        await standardToken.connect(owner).approve(mossR.address,"100000000000000000000");

        await mossR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",34434)

        expect(await mapVault.vaultBalance(34434)).to.equal("-1000000000000000000")
        expect(await standardToken.totalSupply()).to.equal("0");
        console.log(await standardToken.totalSupply());

        await standardToken.mint(owner.address,"1000000000000000000");

        await tokenRegister.registerToken(standardToken.address,mapVault.address, false);

        await mossR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",1313161555)

        expect(await mapVault.vaultBalance(1313161555)).to.equal("-1000000000000000000")
        expect(await standardToken.totalSupply()).to.equal("1000000000000000000");

        expect(await standardToken.balanceOf(owner.address)).to.equal("0");
    });

    it('transferOutNative test ', async function () {

        await mossR.connect(owner).transferOutNative(address2Bytes,1313161555,{value:"100000000000000000"});

        expect(await wrapped.balanceOf(mossR.address)).to.equal("100000000000000000")
    });


    it('transferIn test ', async function () {
        await tokenRegister.registerToken(standardToken.address,mapVault.address,true);
        //await mossR.addAuthToken([standardToken.address]);
        //console.log(await tokenRegister.getTargetToken(1313161555,212))

        console.log(await usdt.balanceOf(mossR.address));
        console.log(await wrapped.balanceOf(mossR.address));
        console.log(await standardToken.balanceOf(mossR.address));

        await usdt.mint(mossR.address,"15000000000000000");
        let near2eth001Data = await mossR.transferIn(1313161555,mosRelayData.near2eth001);
        let near2eth001Receipt = await ethers.provider.getTransactionReceipt(near2eth001Data.hash)

        let near2eth001Decode = ethers.utils.defaultAbiCoder.decode(['bytes','bytes','bytes32','uint256','uint256','bytes','uint256','bytes'],
            near2eth001Receipt.logs[1].data)

        expect(near2eth001Decode[6]).to.equal("75000000000000000");


        // amount: 150000000000000000000000,
        let near2ethWData = await mossR.transferIn(1313161555,mosRelayData.near2ethW);
        let near2ethWReceipt = await ethers.provider.getTransactionReceipt(near2ethWData.hash)
        let near2ethWDecode = ethers.utils.defaultAbiCoder.decode(['bytes','bytes','bytes32','uint256','uint256','bytes','uint256','bytes'],
            near2ethWReceipt.logs[2].data)
        //console.log(near2ethWDecode)
        expect(near2ethWDecode[6]).to.equal("150000000000000000");

        expect(await mapVault.vaultBalance(34434)).to.equal("-1150000000000000000")
        expect(await mapVault.vaultBalance(1313161555)).to.equal("-850000000000000000")

        //amount: 150000000000000000000000,
        let near2eth000Data =  await mossR.transferIn(1313161555,mosRelayData.near2eth000);
        let near2eth000Receipt = await ethers.provider.getTransactionReceipt(near2eth000Data.hash)

        let near2eth000Decode = ethers.utils.defaultAbiCoder.decode(['bytes','bytes','bytes32','uint256','uint256','bytes','uint256','bytes'],
            near2eth000Receipt.logs[0].data)
        //console.log(near2eth000Decode)
        expect(near2eth000Decode[6]).to.equal("150000000000000000");

        expect(await usdt.balanceOf(mossR.address)).to.equal("0");
        await usdt.mint(mossR.address,"150000000000000000");
        await mossR.transferIn(1313161555,mosRelayData.near2map001);
        expect(await usdt.balanceOf(mossR.address)).to.equal("0")

        await mossR.transferIn(1313161555,mosRelayData.near2mapW);
        expect(await standardToken.totalSupply()).to.equal("1150000000000000000");

        expect(await mapVault.vaultBalance(34434)).to.equal("-1150000000000000000")
        expect(await mapVault.vaultBalance(1313161555)).to.equal("-700000000000000000")

        expect(await wrapped.balanceOf(mossR.address)).to.equal("100000000000000000");
        await wrapped.deposit({value:"50000000000000000"});
        await wrapped.transfer(mossR.address,"50000000000000000");
        await mossR.transferIn(1313161555,mosRelayData.near2map000);
        expect(await wrapped.balanceOf(mossR.address)).to.equal("0");

    });

    it('eth2map transferIn test', async function () {
        expect(await usdt.balanceOf(mossR.address)).to.equal("0");
        await usdt.mint(mossR.address,"100000000000000000000");
        await mossR.transferIn(34434,mosRelayData.eth2mapMapToken);
        expect(await usdt.balanceOf(mossR.address)).to.equal("0")

        //300000000000000000000
        await mossR.transferIn(34434,mosRelayData.eth2mapStandardToken);
        expect(await standardToken.totalSupply()).to.equal("301150000000000000000");

        expect(await mapVault.vaultBalance(34434)).to.equal("298850000000000000000")

        expect(await wrapped.balanceOf(mossR.address)).to.equal("0");
        await wrapped.deposit({value:"2000000000000000000"});
        await wrapped.transfer(mossR.address,"2000000000000000000");
        await mossR.transferIn(34434,mosRelayData.eth2mapNative);
        expect(await wrapped.balanceOf(mossR.address)).to.equal("0");
    });


    it('error test', async function () {

        //assert.equal(await mossR.transferIn(888,mosRelayData.near2eth000),"fail");
        await expect(mossR.transferIn(888,mosRelayData.near2eth000)).to.be.revertedWith("fail")

    });

    it('set test', async function () {
        console.log(await mossR.getAdmin());
        await mossR.setPause();
        expect(await mossR.paused()).to.equal(true);
        await mossR.setUnpause();
        expect(await mossR.paused()).to.equal(false);

        await expect(mossR.connect(addr3).setPause()).to.be.revertedWith("mosRelay :: only admin")

    });

    it('admin test', async function () {

        await expect(mossR.changeAdmin("0x0000000000000000000000000000000000000000")).to.be.revertedWith("address is zero")

        await mossR.changeAdmin(addr5.address);

        expect(await mossR.getAdmin()).to.equal(addr5.address);

    });




    it('collectChainFee test', async function () {
        await usdt.mint(owner.address,"1000000000000000000");
        await usdt.connect(owner).approve(mossR.address,"100000000000000000000");
        await mossR.connect(owner).transferOutToken(usdt.address,address2Bytes,"1000000000000000000",34434);

        expect(await usdt.balanceOf(mossR.address)).to.be.equal("900000000000000000");
        //expect(await mapVaultU.correspondBalance()).to.be.equal("350000000000000000");
        expect(await usdt.balanceOf(addr3.address)).to.be.equal("115000000000000000");

        // set standToken to 34434 fee rate 50%
        await tokenRegister.setTokenFee(standardToken.address,34434,"1000000000000000","2000000000000000000","500000")

        await mossR.connect(addr5).setDistributeRate(0,mossR.address,"400000")
        await mossR.connect(addr5).setDistributeRate(1,addr3.address,"200000")

        console.log(await standardToken.balanceOf(mossR.address));
        await standardToken.mint(owner.address,"1000000000000000000");
        await standardToken.connect(owner).approve(mossR.address,"100000000000000000000");
        await mossR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",34434);

        // to vault 200000000000000000
        //expect(await mapVault.correspondBalance()).to.be.equal("10000200000000000000000");
        // to addr3 100000000000000000
        expect(await standardToken.balanceOf(addr3.address)).to.be.equal("100000000000000000");
        //fee 500000000000000000
        // no processing 200000000000000000 + to vault 200000000000000000
        //400000000000000000
        expect(await standardToken.balanceOf(mossR.address)).to.be.equal("1400000000000000000");



    });

    it(' depositToken and  depositNative test', async function () {
        await standardToken.mint(addr7.address,"10000000000000000000000")

        await standardToken.connect(addr7).approve(mossR.address,"10000000000000000000000")
        await mossR.connect(addr7).depositToken(standardToken.address,addr7.address,"10000000000000000000000")

        console.log(await standardToken.balanceOf(mossR.address));

        //10000200000000000000000
        console.log(await mapVault.totalVault());
        console.log(await mapVault.balanceOf(addr7.address))

        await mossR.connect(addr8).depositNative(addr8.address,{value:"2000000000000000000"})

    });


    it('withdraw test', async function () {
        console.log(await ethers.provider.getBalance(mossR.address));

        await wrapped.connect(addr4).deposit({value:"1000000000000000000"});
        await wrapped.connect(addr4).transfer(mossR.address,"1000000000000000000");

        await mossR.connect(addr5).emergencyWithdraw(
            wrapped.address,
            addr6.address,
            "1000000000000000000"
        )
        expect(await wrapped.balanceOf(mossR.address)).to.equal("2000000000000000000");
        expect(await ethers.provider.getBalance(addr6.address)).to.equal("10001000000000000000000");

        console.log(await standardToken.balanceOf(addr7.address))
        console.log(await mapVault.totalSupply());
       // console.log(await mapVault.correspondBalance());
        //console.log(await mapVault.getCorrespondQuantity("10000000000000000000000"))
        await mapVault.connect(addr7).approve(mossR.address,"10000000000000000000000")

        console.log(standardToken.address)
        //10000000000000000000000
        //1400000000000000000
        console.log(await standardToken.balanceOf(mossR.address));
        //10000200000000000000000
        await mossR.connect(addr7).withdraw(
            mapVault.address,
            "10000000000000000000000"
        )
        expect(await mapVault.balanceOf(addr7.address)).to.equal("0")
        expect(await standardToken.balanceOf(addr7.address)).to.equal("10000200000000000000000")
        expect(await mapVault.totalSupply()).to.equal("0");
        expect(await standardToken.balanceOf(mossR.address)).to.equal("1200000000000000000");

        await mossR.connect(addr5).emergencyWithdraw(
            standardToken.address,
            addr6.address,
            "200000000000000000"
        )

        expect(await standardToken.balanceOf(addr6.address)).to.equal("200000000000000000");

    });

    it('depositIn test ', async function () {
        expect(await usdt.balanceOf(mossR.address)).to.equal("900000000000000000")
        expect(await standardToken.balanceOf(mossR.address)).to.equal("1000000000000000000")
        expect(await wrapped.balanceOf(mossR.address)).to.equal("2000000000000000000")
        expect(await usdt.totalSupply()).to.equal("101165000000000000000")
        expect(await standardToken.totalSupply()).to.equal("10301650000000000000000")

        expect(await mapVault.totalSupply()).to.equal("0");
        expect(await mapVaultU.totalSupply()).to.equal("0");
        expect(await mapVaultW.totalSupply()).to.equal("2000000000000000000");
        //0x6d63735f746f6b656e5f302e6d63732e6d61703030312e746573746e6574
        //100000000000000000
        await expect(mossR.depositIn("5566818579631833089",mosRelayData.near2mapDepositeM01E)).to.be.revertedWith("invalid chain id");


        await mossR.depositIn("1313161555",mosRelayData.near2mapDepositeM01);

        expect(await usdt.balanceOf(mossR.address)).to.equal("900000000000000000")
        expect(await standardToken.balanceOf(mossR.address)).to.equal("1000000000000000000")
        expect(await wrapped.balanceOf(mossR.address)).to.equal("2000000000000000000")
        expect(await usdt.totalSupply()).to.equal("101165000000000000000")
        expect(await standardToken.totalSupply()).to.equal("10301650000000000000000")

        expect(await mapVault.totalSupply()).to.equal("0");
        expect(await mapVaultU.totalSupply()).to.equal("100000000000000000");
        expect(await mapVaultW.totalSupply()).to.equal("2000000000000000000");
        //
        // await mossR.setmosContract(34434,"0xAC25DeA31A410900238c8669eD9973f328919160",1);
        //0x777261702e746573746e6574
        //100000000000000000
        // await mossR.depositIn("5566818579631833089",mosRelayData.near2mapDepositeM01);
        // expect(await mapVault.totalSupply()).to.equal("100000000000000000");
        //
        // expect(await standardToken.totalSupply()).to.equal("10301750000000000000000")

        //0xd293ffec6c0ed1abda02a72ac0199858cd5cc4a9
        await mossR.connect(addr5).registerChain(97,"0xd293ffec6c0ed1abda02a72ac0199858cd5cc4a9",1);

        //satand token
        //100000000000000000
        await mossR.depositIn(97,mosRelayData.eth2mapDepositeU);
        // expect(await standardToken.totalSupply()).to.equal("10301150000000000000000");
        // expect(await mapVault.balanceOf("0x2e784874ddb32cd7975d68565b509412a5b519f4")).to.equal("10000000000000000000000")
        // expect(await mapVault.totalSupply()).to.equal("10000000000000000000000");
        expect(await usdt.balanceOf(mossR.address)).to.equal("900000000000000000")
        expect(await standardToken.balanceOf(mossR.address)).to.equal("1000000000000000000")
        expect(await usdt.totalSupply()).to.equal("101165000000000000000")
        expect(await mapVaultU.totalSupply()).to.equal("30403030303030303030");

    });


    it('Upgrade', async function () {
        let MOSSRelayUpGrade = await ethers.getContractFactory("MAPOmnichainServiceRelayV2");
        // moss = await ethers.getContractAt("MapCrossChainService",mosData.mos);
        let mossRUpGrade = await MOSSRelayUpGrade.deploy();
        await mossRUpGrade.deployed();

        mossR.connect(addr5).upgradeTo(mossRUpGrade.address);

        expect(await mossR.getImplementation()).to.equal(mossRUpGrade.address);

        await expect(mossR.transferIn(1313161555,mosRelayData.near2mapW)).to.be.revertedWith("order exist");

        // await mossR.connect(addr5).setLightClientManager(addr4.address);
        // expect(await mossR.lightClientManager()).to.be.equal(addr4.address);
    });


    it('deposit and withdraw ', async function () {

        //200000000000000000000
        await mossR.depositIn(97,mosRelayData.eth2mapDepositeS);
        expect(await standardToken.balanceOf(mossR.address)).to.equal("201000000000000000000")
        expect(await mapVault.balanceOf(addr8.address)).to.equal("200000000000000000000")
        expect(await standardToken.totalSupply()).to.equal("10501650000000000000000")
        expect(await mapVault.totalSupply()).to.equal("200000000000000000000");


        //1000000000000000
        await mossR.depositIn(97,mosRelayData.eth2mapDepositeW);
        expect(await wrapped.balanceOf(mossR.address)).to.equal("2000000000000000000")
        expect(await mapVaultW.totalSupply()).to.equal("2001000000000000000");

        await standardToken.mint(addr2.address,"20000000000000000000");
        await standardToken.connect(addr2).approve(mossR.address,"2000000000000000000000");
        await mossR.connect(addr2).transferOutToken(standardToken.address,address2Bytes,"20000000000000000000",34434)
        //2008
        expect(await mapVault.totalVault()).to.equal("200800000000000000000");
        expect(await mapVault.totalSupply()).to.equal("200000000000000000000");
        await standardToken.mint(addr1.address,"1000000000000000000");
        await standardToken.connect(addr1).approve(mossR.address,"100000000000000000000");
        await mossR.connect(addr1).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",34434)
        //2010
        expect(await mapVault.totalVault()).to.equal("201000000000000000000");
        expect(await mapVault.totalSupply()).to.equal("200000000000000000000");
        console.log(await standardToken.balanceOf(addr7.address));
        await standardToken.connect(addr7).approve(mossR.address,"10000000000000000000000");
        await mossR.connect(addr7).depositToken(standardToken.address,addr7.address,"10000000000000000000000")
        expect(await mapVault.totalVault()).to.equal("10201000000000000000000");

        //100000 * 2000 / 2010 = 99502 + 2000 = 101502
        expect(await mapVault.totalSupply()).to.equal("10150248756218905472636");

        expect(await standardToken.balanceOf(addr8.address)).to.equal("0");

        await mapVault.connect(addr8).approve(mossR.address,"200000000000000000000")
        await mossR.connect(addr8).withdraw(mapVault.address,"200000000000000000000")

        expect(await mapVault.balanceOf(addr8.address)).to.equal("0");
        //200000000000000000000 + 1000000000000000000(fee)
        expect(await standardToken.balanceOf(addr8.address)).to.equal("201000000000000000000");


    });


})
