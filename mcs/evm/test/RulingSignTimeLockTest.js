const {ethers} = require("hardhat");
const {expect} = require("chai");
const Web3 = require('web3')


describe("RulingSignTimeLock start test", function () {

    let proposer1;
    let proposer2;
    let proposer3;

    let mcss;

    let rulingSignTimeLock;
    let web3;


    beforeEach(async function () {
        [deployer, proposer1, proposer2, proposer3] = await ethers.getSigners();
        web3 = new Web3("")
    });


    it("MapCrossChainService deploy init", async function () {
        console.log("deployer", deployer.address)
        console.log("proposer1",proposer1.address)
        console.log("proposer2",proposer2.address)
        console.log("proposer3",proposer3.address)

        console.log("deployer address:",deployer.address);
        MCSS = await ethers.getContractFactory("MapCrossChainService");
        mcss = await MCSS.deploy();
        console.log("mcss address:",mcss.address);
        StandardToken = await ethers.getContractFactory("StandardToken");

        standardToken = await  StandardToken.deploy("MapToken","MP");

        await mcss.initialize(standardToken.address,standardToken.address,standardToken.address);

    });

    it("RulingSignTimeLock deploy init", async function () {
        RulingSignTimeLock = await ethers.getContractFactory("RulingSignTimeLock");
        rulingSignTimeLock = await RulingSignTimeLock.deploy([proposer1.address,proposer2.address,proposer3.address],2,1000);
        console.log("rulingSignTimeLock address:",rulingSignTimeLock.address);

        mcss.transferOwnership(rulingSignTimeLock.address);
        console.log("change owner to:",rulingSignTimeLock.address);

    });


    it("RulingSignTimeLock vote setCanBridgeToken", async function () {
        let calldata = web3.eth.abi.encodeFunctionCall(  {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "token",
                    "type": "address"
                },
                {
                    "internalType": "uint256",
                    "name": "chainId",
                    "type": "uint256"
                },
                {
                    "internalType": "bool",
                    "name": "canBridge",
                    "type": "bool"
                }
            ],
            "name": "setCanBridgeToken",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },["0x0000000000000000000000000000000000000008",1,true])

        console.log("call setCanBridgeToken data:", calldata)

        await rulingSignTimeLock.connect(proposer1).submitTransactionCall(mcss.address
            ,0
            ,1
            ,calldata);

        let id = await rulingSignTimeLock.hashOperation(mcss.address,
            0,
            calldata,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000")

        console.log("add proposer add vote")

        await rulingSignTimeLock.connect(proposer2).confirmTransaction(0);

        console.log("add vote and to time lock")


        const threeDays = 3 * 24 * 60 * 60;

        const blockNumBefore = await ethers.provider.getBlockNumber();
        const blockBefore = await ethers.provider.getBlock(blockNumBefore);
        const timestampBefore = blockBefore.timestamp;

        console.log("timestampBefore",timestampBefore)

        await ethers.provider.send('evm_increaseTime', [threeDays]);
        await ethers.provider.send('evm_mine');

        const blockNumAfter = await ethers.provider.getBlockNumber();
        const blockAfter = await ethers.provider.getBlock(blockNumAfter);
        const timestampAfter = blockAfter.timestamp;

        console.log("blockNumAfter",timestampAfter)
        console.log("time",await rulingSignTimeLock.getTimestamp(id))

        await rulingSignTimeLock.connect(proposer1).execute(mcss.address,
            0,
            calldata,
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000000000000000000000000000000000000000000000");


    });


})