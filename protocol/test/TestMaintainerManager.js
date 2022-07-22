const hre = require('hardhat');
const {chai} = require('chai');
const {ethers} = require('hardhat');
const {BigNumber} = require("ethers");
const expect = chai.expect;


describe('MaintainerManager', function () {
    let mm;
    let owner;
    let user1;
    before(async () => {
        let MaintainerManager = await hre.ethers.getContractFactory("MaintainerManager");
        mm = await MaintainerManager.deploy();
        await mm.deployed();
        owner = await ethers.getSigners();
        console.log(mm.address)

        let signers = await ethers.getSigners();
        this.deployer = signers[0];
        this.user1 = signers[1];
    });


    it("should verify maximum quorum", async () => {
        await mm.addWhiteList(this.deployer.address); // 1111
        await mm.deposit({value:"1"});
        await mm.addAward("1");
        console.log(mm.pendingReward(this.deployer.address));

        await mm.addAward("10");
        console.log(await mm.pendingReward(this.deployer.address));

        await mm.addWhiteList(this.user1.address);
        await mm.connect(this.user1).deposit({value:"1"});
        console.log(await mm.pendingReward(this.user1.address));
        await mm.addAward("2")
        console.log(await mm.pendingReward(this.deployer.address));
        console.log(await mm.pendingReward(this.user1.address));
    });



});
