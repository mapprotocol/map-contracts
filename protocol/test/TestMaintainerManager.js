const hre = require('hardhat');
const {chai} = require('chai');
const {ethers} = require('hardhat');
const {BigNumber} = require("ethers");
const {assert} = require('chai');

// const expect = chai.expect;


describe('MaintainerManager', function () {
    let mm;
    let owner;
    let user1;
    before(async () => {
        let MaintainerManager = await hre.ethers.getContractFactory("MaintainerManager");
        mm = await MaintainerManager.deploy();
        await mm.deployed();
        mm.initialize();
        owner = await ethers.getSigners();
        console.log(mm.address)

        let signers = await ethers.getSigners();
        this.deployer = signers[0];
        user1 = signers[1];
    });


    it("should verify maximum quorum", async () => {
        await mm.addWhiteList(this.deployer.address); // 1111
        console.log(await mm.getAllAwards());

        assert.equal(await mm.getAllAwards(),0);

        await mm.deposit({value:"1"});

        await mm.save({value:"1"});

        assert.equal(await mm.pendingReward(this.deployer.address),1);

        await mm.save({value:"10"});

        assert.equal(await mm.pendingReward(this.deployer.address),11);

        await mm.addWhiteList(user1.address);
        console.log("addWhiteList is ok");
        await mm.connect(user1).deposit({value:"100"});

        console.log("user1 deposit ok");

        assert.equal(await mm.pendingReward(this.deployer.address),11);
        assert.equal(await mm.pendingReward(user1.address),0);

        await mm.save({value:"101"});

        assert.equal(await mm.pendingReward(this.deployer.address),12);
        assert.equal(await mm.pendingReward(user1.address),100);

        await mm.withdraw(1);

        console.log(await mm.userInfo(this.deployer.address))
        console.log(await mm.userInfo(user1.address))
        await mm.connect(user1).withdraw(100);


        await mm.save({value:"101"});

        assert.equal(await mm.pendingReward(this.deployer.address),0);
        assert.equal(await mm.pendingReward(user1.address),0);

        await mm.deposit({value:"1"});
        assert.equal(await mm.pendingReward(this.deployer.address),101);

        await mm.connect(user1).deposit({value:"1"});
        await mm.save({value:"100"});

        assert.equal(await mm.pendingReward(this.deployer.address),151);
        assert.equal(await mm.pendingReward(user1.address),50);
    });



});
