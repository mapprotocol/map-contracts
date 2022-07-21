const hre = require('hardhat');
const {assert} = require('chai');
const {ethers} = require('hardhat');
const {BigNumber} = require("ethers");


describe('MaintainerManager', function () {
    let mm;
    let owner;

    before(async () => {
        let MaintainerManager = await hre.ethers.getContractFactory("MaintainerManager");
        mm = await MaintainerManager.deploy("0","10000000000000");
        await mm.deployed();
        owner = await ethers.getSigners();
        console.log(mm.address)
        const { deployer} = await ethers.getNamedSigners()
        this.deployer = deployer;
    });


    it("should verify maximum quorum", async () => {
        await mm.addWhiteList(this.deployer.address); // 1111
        await mm.deposit({value:"1"});
        await mm.withdraw("1");
    });



});
