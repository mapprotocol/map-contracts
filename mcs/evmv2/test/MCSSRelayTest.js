const { ethers } = require("hardhat");
const { expect } = require("chai");
const mcsRelayData = require('./mcsRelayData');
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

    let EvmDecoder;
    let evmDecoder;

    let NearDecoder;
    let nearDecoder;

    let MCSSRelay;
    let mcssR;

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

        [deployer,owner, addr1, addr2, addr3, addr4, addr5,addr6,addr7] = await ethers.getSigners();

    });

    it("MAPOmnichainServiceRelayV2 contract deploy init", async function () {
        console.log("deployer address:",deployer.address)


        EvmDecoder = await ethers.getContractFactory("EvmDecoder");
        evmDecoder = await EvmDecoder.deploy();
        console.log("EvmDecoder address:",evmDecoder.address);

        NearDecoder = await ethers.getContractFactory("NearDecoder");
        nearDecoder = await NearDecoder.deploy();
        console.log("NearDecoder address:",nearDecoder.address);

        MCSSRelay = await ethers.getContractFactory("MAPOmnichainServiceRelayV2");
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mcssR = await MCSSRelay.deploy();
        console.log("mcssR address:",mcssR.address);

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
        // mcss = await ethers.getContractAt("MapCrossChainService",mcsData.mcs);
        mapVault = await MapVault.deploy(standardToken.address,"MapVaultToken","MVT");
        console.log("MapVault  address:",mapVault.address);
        //await mapVault.initialize(standardToken.address,"MapVaultToken","MVT","18");

        MapVaultU = await ethers.getContractFactory("VaultTokenV2");
        mapVaultU = await MapVaultU.deploy(usdt.address,"MapVaultTokenUsdt","UVT");

        MapVaultW = await ethers.getContractFactory("VaultTokenV2");
        mapVaultW = await MapVaultU.deploy(wrapped.address,"MapVaultTokenWrapped","WVT");

        //await mapVaultU.initialize(usdt.address,"MapVaultTokenUsdt","UVT","18");

        let data = await mcssR.initialize(wrapped.address,lightClientManager.address);
        initData = data.data;
    });

    it('UUPS test', async function () {
        const MAPCrossChainServiceRelayProxy = await ethers.getContractFactory("MAPOmnichainServiceProxyV2");
        let mcssRP = await MAPCrossChainServiceRelayProxy.deploy(mcssR.address,initData);
        await mcssRP.deployed()

        mcssR = MCSSRelay.attach(mcssRP.address);

    });

    it('mcsRelay contract set ', async function () {
        await mcssR.setTokenManager(tokenRegister.address);

        expect(await mcssR.tokenRegister()).to.equal(tokenRegister.address);

        await mcssR.registerChain(34434,mcsRelayData.mcsETH,1);

        await mcssR.registerChain(1313161555,mcsRelayData.mcsNear,2);

        expect(await mcssR.chainTypes(34434)).to.equal(1)

        await mapVault.addManager(mcssR.address);
        await mapVaultU.addManager(mcssR.address);
        await mapVaultW.addManager(mcssR.address);

        await mcssR.setDistributeRate(0,addr2.address,"400000")
        await mcssR.setDistributeRate(1,addr3.address,"200000")
        //expect(await mcssR.checkAuthToken(standardToken.address)).to.equal("true");
    });

    it('TokenRegister set', async function () {
        await tokenRegister.registerToken(usdt.address,mapVaultU.address,false);
        await tokenRegister.registerToken(standardToken.address,mapVault.address,true);
        await tokenRegister.registerToken(wrapped.address,mapVaultW.address,false);

        await tokenRegister.mapToken(usdt.address,34434,mcsRelayData.ethUsdtToken,18);
        await tokenRegister.mapToken(standardToken.address,34434,mcsRelayData.ethStanardToken,18);
        await tokenRegister.mapToken(usdt.address,212,usdt.address,18);
        await tokenRegister.mapToken(standardToken.address,212,standardToken.address,18);
        await tokenRegister.mapToken(usdt.address,1313161555,mcsRelayData.nearUsdtToken,24);
        await tokenRegister.mapToken(standardToken.address,1313161555,mcsRelayData.nearWethToken,24);
        await tokenRegister.mapToken(wrapped.address,1313161555,"0x0000000000000000000000000000000000000000",24);
        await tokenRegister.mapToken(wrapped.address,212,wrapped.address,18);
        await tokenRegister.mapToken(wrapped.address,34434,"0x0000000000000000000000000000000000000000",18);

        await tokenRegister.setTokenFee(usdt.address,34434,"1000000000000000","2000000000000000000","500000")

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

        await tokenRegister.registerToken(standardToken.address,mapVault.address, false);

        await mcssR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",1313161555)

        expect(await standardToken.totalSupply()).to.equal("1000000000000000000");

        expect(await standardToken.balanceOf(owner.address)).to.equal("0");
    });

    it('transferOutNative test ', async function () {

        await mcssR.connect(owner).transferOutNative(address2Bytes,1313161555,{value:"100000000000000000"});

        expect(await wrapped.balanceOf(mcssR.address)).to.equal("100000000000000000")
    });


    it('transferIn test ', async function () {
        await tokenRegister.registerToken(standardToken.address,mapVault.address,true);
        //await mcssR.addAuthToken([standardToken.address]);
        //console.log(await tokenRegister.getTargetToken(1313161555,212))

        console.log(await usdt.balanceOf(mcssR.address));
        console.log(await wrapped.balanceOf(mcssR.address));
        console.log(await standardToken.balanceOf(mcssR.address));

        await usdt.mint(mcssR.address,"15000000000000000");
        let near2eth001Data = await mcssR.transferIn(1313161555,mcsRelayData.near2eth001);
        let near2eth001Receipt = await ethers.provider.getTransactionReceipt(near2eth001Data.hash)

        let near2eth001Decode = ethers.utils.defaultAbiCoder.decode(['bytes','bytes','bytes32','uint256','uint256','bytes','uint256','bytes'],
            near2eth001Receipt.logs[1].data)

        expect(near2eth001Decode[6]).to.equal("75000000000000000");

        // amount: 150000000000000000000000,
        let near2ethWData = await mcssR.transferIn(1313161555,mcsRelayData.near2ethW);
        let near2ethWReceipt = await ethers.provider.getTransactionReceipt(near2ethWData.hash)
        let near2ethWDecode = ethers.utils.defaultAbiCoder.decode(['bytes','bytes','bytes32','uint256','uint256','bytes','uint256','bytes'],
            near2ethWReceipt.logs[2].data)
        //console.log(near2ethWDecode)
        expect(near2ethWDecode[6]).to.equal("150000000000000000");

        //amount: 150000000000000000000000,
        let near2eth000Data =  await mcssR.transferIn(1313161555,mcsRelayData.near2eth000);
        let near2eth000Receipt = await ethers.provider.getTransactionReceipt(near2eth000Data.hash)

        let near2eth000Decode = ethers.utils.defaultAbiCoder.decode(['bytes','bytes','bytes32','uint256','uint256','bytes','uint256','bytes'],
            near2eth000Receipt.logs[0].data)
        //console.log(near2eth000Decode)
        expect(near2eth000Decode[6]).to.equal("150000000000000000");

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

    // it('depositIn test ', async function () {
    //     // expect(await usdt.balanceOf(mcssR.address)).to.equal("0")
    //     //await feeCenter.setTokenVault(usdt.address,mapVaultU.address)
    //     expect(await mapVaultU.totalSupply()).to.equal("0");
    //     await usdt.mint(mcssR.address,"150000000000000000");
    //     console.log(await usdt.balanceOf(mcssR.address));
    //     await mcssR.depositIn(1313161555,mcsRelayData.near2mapDeposite);
    //     console.log("1111111111")
    //     expect(await usdt.balanceOf(mcssR.address)).to.equal("0")
    //     expect(await mapVaultU.balanceOf("0x2e784874ddb32cd7975d68565b509412a5b519f4")).to.equal("150000000000000000")
    //     expect(await mapVaultU.totalSupply()).to.equal("150000000000000000");
    //
    //     await mcssR.setMcsContract(34434,"0xAC25DeA31A410900238c8669eD9973f328919160",1);
    //
    //
    //     await mcssR.depositIn(34434,mcsRelayData.eth2mapDeposite);
    //     console.log("2222222222")
    //     expect(await standardToken.totalSupply()).to.equal("10301150000000000000000");
    //     expect(await mapVault.balanceOf("0x2e784874ddb32cd7975d68565b509412a5b519f4")).to.equal("10000000000000000000000")
    //     expect(await mapVault.totalSupply()).to.equal("10000000000000000000000");
    // });


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

        await expect(mcssR.changeAdmin("0x0000000000000000000000000000000000000000")).to.be.revertedWith("address is zero")

        await mcssR.changeAdmin(addr5.address);

        expect(await mcssR.getAdmin()).to.equal(addr5.address);

    });

    it('Upgrade', async function () {
        let MCSSRelayUpGrade = await ethers.getContractFactory("MAPOmnichainServiceRelayV2");
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

        expect(await usdt.balanceOf(mcssR.address)).to.be.equal("900000000000000000");
        //expect(await mapVaultU.correspondBalance()).to.be.equal("350000000000000000");
        expect(await usdt.balanceOf(addr3.address)).to.be.equal("115000000000000000");

        // set standToken to 34434 fee rate 50%
        await tokenRegister.setTokenFee(standardToken.address,34434,"1000000000000000","2000000000000000000","500000")

        await mcssR.connect(addr5).setDistributeRate(0,mcssR.address,"400000")
        await mcssR.connect(addr5).setDistributeRate(1,addr3.address,"200000")

        console.log(await standardToken.balanceOf(mcssR.address));
        await standardToken.mint(owner.address,"1000000000000000000");
        await standardToken.connect(owner).approve(mcssR.address,"100000000000000000000");
        await mcssR.connect(owner).transferOutToken(standardToken.address,address2Bytes,"1000000000000000000",34434);

        // to vault 200000000000000000
        //expect(await mapVault.correspondBalance()).to.be.equal("10000200000000000000000");
        // to addr3 100000000000000000
        expect(await standardToken.balanceOf(addr3.address)).to.be.equal("100000000000000000");
        //fee 500000000000000000
        // no processing 200000000000000000 + to vault 200000000000000000
        //400000000000000000
        expect(await standardToken.balanceOf(mcssR.address)).to.be.equal("1400000000000000000");

        await mcssR.connect(addr5).setLightClientManager(addr4.address);
        expect(await mcssR.lightClientManager()).to.be.equal(addr4.address);

    });

    it(' depositToken and  depositNative test', async function () {
        await standardToken.mint(addr7.address,"10000000000000000000000")

        await standardToken.connect(addr7).approve(mcssR.address,"10000000000000000000000")
        await mcssR.connect(addr7).depositToken(standardToken.address,addr7.address,"10000000000000000000000")

        console.log(await standardToken.balanceOf(mcssR.address));

        //10000200000000000000000
        console.log(await mapVault.totalVault());
        console.log(await mapVault.balanceOf(addr7.address))
    });


    it('withdraw test', async function () {
        console.log(await ethers.provider.getBalance(mcssR.address));

        await wrapped.connect(addr4).deposit({value:"1000000000000000000"});
        await wrapped.connect(addr4).transfer(mcssR.address,"1000000000000000000");

        await mcssR.connect(addr5).emergencyWithdraw(
            wrapped.address,
            addr6.address,
            "1000000000000000000"
        )
        expect(await wrapped.balanceOf(mcssR.address)).to.equal("0");
        expect(await ethers.provider.getBalance(addr6.address)).to.equal("10001000000000000000000");

        console.log(await standardToken.balanceOf(addr7.address))
        console.log(await mapVault.totalSupply());
       // console.log(await mapVault.correspondBalance());
        //console.log(await mapVault.getCorrespondQuantity("10000000000000000000000"))
        await mapVault.connect(addr7).approve(mcssR.address,"10000000000000000000000")

        console.log(standardToken.address)
        //10000000000000000000000
        //1400000000000000000
        console.log(await standardToken.balanceOf(mcssR.address));
        //10000200000000000000000
        await mcssR.connect(addr7).withdraw(
            mapVault.address,
            "10000000000000000000000"
        )
        expect(await mapVault.balanceOf(addr7.address)).to.equal("0")
        expect(await standardToken.balanceOf(addr7.address)).to.equal("10000200000000000000000")
        expect(await mapVault.totalSupply()).to.equal("0");
        expect(await standardToken.balanceOf(mcssR.address)).to.equal("1200000000000000000");

        await mcssR.connect(addr5).emergencyWithdraw(
            standardToken.address,
            addr6.address,
            "200000000000000000"
        )

        expect(await standardToken.balanceOf(addr6.address)).to.equal("200000000000000000");
    });
})
