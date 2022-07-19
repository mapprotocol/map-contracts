const {
  time,
  loadFixture,
} = require("@nomicfoundation/hardhat-network-helpers");
const { anyValue } = require("@nomicfoundation/hardhat-chai-matchers/withArgs");
const { expect } = require("chai");
const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('./utils/borsh');
const hre = require("hardhat");
const bs58 = require('bs58');
const { ethers } = require("hardhat");




let nearProofProducerAccount_ = "0x6175726f7261";

describe("lightNode", function () {

  async function deployLightNode() {
    let [wallet] = await ethers.getSigners();

    const LightNode = await ethers.getContractFactory("LightNode");
    const lightNode = await LightNode.connect(wallet).deploy();
    await lightNode.deployed();

    await lightNode.initialize(nearProofProducerAccount_);

    return lightNode;
  }

  describe("Deployment", function () {


    it("initWithValidators must owner", async function () {

      let [wallet, other] = await ethers.getSigners();

      let lightNode = await loadFixture(deployLightNode);

      let block120998 = borshify(require('./data/block_120998.json'));

      let validators = borshifyInitialValidators(require('./data/block_120998.json').next_bps);

      await expect(lightNode.connect(other).initWithValidators(validators)).to.be.reverted;

    });


    it("initWithBlock must owner", async function () {

      let [wallet, other] = await ethers.getSigners();

      let lightNode = await loadFixture(deployLightNode);

      let block120998 = borshify(require('./data/block_120998.json'));

      let validators = borshifyInitialValidators(require('./data/block_120998.json').next_bps);

      await lightNode.connect(wallet).initWithValidators(validators);

      await expect(lightNode.connect(other).initWithBlock(block120998)).to.be.reverted;

    });

    it("should be ok", async function () {

      let [wallet] = await ethers.getSigners();

      let lightNode = await loadFixture(deployLightNode);

      let block120998 = borshify(require('./data/block_120998.json'));

      let validators = borshifyInitialValidators(require('./data/block_120998.json').next_bps);

      await lightNode.connect(wallet).initWithValidators(validators);

      await lightNode.connect(wallet).initWithBlock(block120998);

      expect(await lightNode.blockHashes_(120998)).to.be.equal('0x1a7a07b5eee1f4d8d7e47864d533143972f858464bacdc698774d167fb1b40e6');

    });


  });
});
