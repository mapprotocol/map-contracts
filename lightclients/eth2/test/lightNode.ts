import {loadFixture} from "@nomicfoundation/hardhat-network-helpers";
import {expect} from "chai";
import {Contract} from "ethers";
import {ethers} from "hardhat";
import config from "../hardhat.config";
import {delay} from "../utils/Util";

let bootstrap = require("../data/goerli/bootstrap.js");
let periodUpdate = require("../data/goerli/period_update.js");
let exedata = require("../data/goerli/exe_headers.js");
let chainId = 5; //test data from eth testnet

// let bootstrap = require("../data/mainnet/bootstrap.js");
// let periodUpdate = require("../data/mainnet/period_update.js");
// let exedata = require("../data/mainnet/exe_headers.js");
// let chainId = 1; //test data from eth mainnet

describe("LightNode", function () {
    // We define a fixture to reuse the same setup in every test.
    // We use loadFixture to run this setup once, snapshot that state,
    // and reset Hardhat Network to that snapshopt in every test.

    if (config.defaultNetwork != "hardhat") {
        return;
    }

    async function deployFixture() {
        let [wallet] = await ethers.getSigners();

        const MPTVerify = await ethers.getContractFactory("MPTVerify");

        const mPTVerify = await MPTVerify.deploy();

        await mPTVerify.connect(wallet).deployed();

        const LightNode = await ethers.getContractFactory("LightNode");

        const lightNode = await LightNode.deploy();

        await lightNode.connect(wallet).deployed();

        const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

        let initData = LightNode.interface.encodeFunctionData("initialize", [
            chainId,
            wallet.address,
            mPTVerify.address,
            bootstrap.finalizedBeaconHeader,
            bootstrap.finalizedExeHeaderNumber,
            bootstrap.finalizedExeHeaderHash,
            bootstrap.curSyncCommitteAggPubkey,
            bootstrap.nextSyncCommitteAggPubkey,
            bootstrap.syncCommitteePubkeyHashes,
            false,
        ]);

        const lightNodeProxy = await LightNodeProxy.deploy(
            lightNode.address,
            initData
        );
        await lightNodeProxy.connect(wallet).deployed();
        let proxy = LightNode.attach(lightNodeProxy.address);

        for (let i = 0; i < bootstrap.curSyncCommitteePubkeys.length; i++) {
            await proxy.initSyncCommitteePubkey(bootstrap.curSyncCommitteePubkeys[i]);
            let initStage = await proxy.initStage();
            expect(initStage).to.eq(2 + i);
            let initialized = await proxy.initialized();
            expect(initialized).false;
        }

        for (let i = 0; i < bootstrap.nextSyncCommitteePubkeys.length; i++) {
            await proxy.initSyncCommitteePubkey(
                bootstrap.nextSyncCommitteePubkeys[i]
            );
            let initStage = await proxy.initStage();
            let initialized = await proxy.initialized();
            if (i != bootstrap.nextSyncCommitteePubkeys.length - 1) {
                expect(initStage).to.eq(5 + i);
                expect(initialized).false;
            }
        }

        return proxy;
    }

    describe("Initialization", function () {
        it("initialization should be OK", async function () {
            let lightNode = await loadFixture(deployFixture);

            let slot = await lightNode.headerHeight();
            expect(slot).to.eq(bootstrap.finalizedBeaconHeader.slot);

            let initStage = await lightNode.initStage();
            expect(initStage).to.eq(6);

            let initialized = await lightNode.initialized();
            expect(initialized).true;
        });

        it("can not initialization sync committee after contract is initialized", async function () {
            let lightNode = await loadFixture(deployFixture);

            let initStage = await lightNode.initStage();
            expect(initStage).to.eq(6);

            let initialized = await lightNode.initialized();
            expect(initialized).true;

            await expect(
                lightNode.initSyncCommitteePubkey(bootstrap.curSyncCommitteePubkeys[0])
            ).to.be.revertedWith("contract is initialized!");
        });

        it("re-initialization should fail", async function () {
            let [wallet] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            let initStage = await lightNode.initStage();
            expect(initStage).to.eq(6);

            let initialized = await lightNode.initialized();
            expect(initialized).true;

            await expect(
                lightNode.initialize(
                    chainId,
                    wallet.address,
                    wallet.address,
                    bootstrap.finalizedBeaconHeader,
                    bootstrap.finalizedExeHeaderNumber,
                    bootstrap.finalizedExeHeaderHash,
                    bootstrap.curSyncCommitteAggPubkey,
                    bootstrap.nextSyncCommitteAggPubkey,
                    bootstrap.syncCommitteePubkeyHashes,
                    false
                )
            ).to.be.revertedWith("Initializable: contract is already initialized");
        });

        it("initialization with wrong sync committee keys should be fail", async function () {
            let [wallet] = await ethers.getSigners();
            const MPTVerify = await ethers.getContractFactory("MPTVerify");
            const mPTVerify = await MPTVerify.deploy();
            await mPTVerify.connect(wallet).deployed();

            const LightNode = await ethers.getContractFactory("LightNode");
            const lightNode = await LightNode.deploy();
            await lightNode.connect(wallet).deployed();

            const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");
            let initData = LightNode.interface.encodeFunctionData("initialize", [
                chainId,
                wallet.address,
                mPTVerify.address,
                bootstrap.finalizedBeaconHeader,
                bootstrap.finalizedExeHeaderNumber,
                bootstrap.finalizedExeHeaderHash,
                bootstrap.curSyncCommitteAggPubkey,
                bootstrap.nextSyncCommitteAggPubkey,
                bootstrap.syncCommitteePubkeyHashes,
                false,
            ]);
            const lightNodeProxy = await LightNodeProxy.deploy(
                lightNode.address,
                initData
            );
            await lightNodeProxy.connect(wallet).deployed();
            let proxy = LightNode.attach(lightNodeProxy.address);

            await expect(
                proxy.initSyncCommitteePubkey(bootstrap.curSyncCommitteePubkeys[1])
            ).to.be.revertedWith("wrong syncCommitteePubkeyPart hash");

            let initialized = await lightNode.initialized();
            expect(initialized).false;
        });
    });

    describe("Upgrade", function () {
        it("Implementation upgrade must be admin", async function () {
            let [wallet, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();
            expect(admin).to.not.eq(other.address);

            const LightNode = await ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            await expect(
                lightNode.connect(other).upgradeTo(newImplement.address)
            ).to.be.revertedWith("LightNode: only Admin can upgrade");
        });

        it("Implementation upgrade is OK", async function () {
            let [wallet, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();
            expect(admin).to.not.eq(other.address);
            expect(admin).to.eq(wallet.address);

            const LightNode = await ethers.getContractFactory("LightNode");
            const newImplement = await LightNode.connect(wallet).deploy();
            await newImplement.deployed();

            let oldImplement = await lightNode.getImplementation();
            expect(oldImplement).to.not.eq(newImplement.address);

            await lightNode.connect(wallet).upgradeTo(newImplement.address);
            expect(await lightNode.getImplementation()).to.eq(newImplement.address);

            let slot = await lightNode.headerHeight();
            expect(slot).to.eq(bootstrap.finalizedBeaconHeader.slot);
        });
    });

    describe("Permission check", function () {
        it("Change admin", async function () {
            let [wallet, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            let admin = await lightNode.getAdmin();
            expect(admin).to.eq(wallet.address);

            await expect(
                lightNode.connect(other).changeAdmin(other.address)
            ).to.be.revertedWith("lightnode :: only admin");

            await expect(
                lightNode.connect(wallet).changeAdmin(ethers.constants.AddressZero)
            ).to.be.revertedWith("zero address");

            await lightNode.connect(wallet).changeAdmin(other.address);
            expect(await lightNode.getAdmin()).to.eq(other.address);
        });

        it("togglePause  only admin ", async function () {
            let [wallet, other] = await ethers.getSigners();
            let lightNode = await loadFixture(deployFixture);

            let paused = await lightNode.paused();
            expect(paused).to.false;

            await expect(
                lightNode.connect(other).togglePause(true)
            ).to.be.revertedWith("lightnode :: only admin");

            await lightNode.connect(wallet).togglePause(true);
            expect(await lightNode.paused()).to.true;

            await lightNode.connect(wallet).togglePause(false);
            expect(await lightNode.paused()).to.false;
        });
    });

    describe("Update light client", function () {
        it("updateLightClient ... paused ", async function () {
            let [wallet] = await ethers.getSigners();
            let lightNode = await deployFixture();

            await lightNode.connect(wallet).togglePause(true);
            let update = await lightNode.getUpdateBytes(periodUpdate.update)

            await expect(
                lightNode.updateLightClient(update)
            ).to.be.revertedWith("Pausable: paused");
        });

        it("updateLightClient ... OK ", async function () {
            let lightNode = await deployFixture();
            let initialized = await lightNode.initialized();
            expect(initialized).true;

            let update = await lightNode.getUpdateBytes(periodUpdate.update)
            await lightNode.updateLightClient(update);

            let finalizedSlot = await lightNode.headerHeight();
            expect(finalizedSlot).to.eq(periodUpdate.update.finalizedHeader.slot);

            let exeHeaderUpdateInfo = await lightNode.exeHeaderUpdateInfo();
            expect(exeHeaderUpdateInfo.startNumber).to.eq(
                bootstrap.finalizedExeHeaderNumber.toNumber() + 1
            );
            expect(exeHeaderUpdateInfo.endNumber).to.eq(
                periodUpdate.update.finalizedExeHeader.number - 1
            );
            expect(exeHeaderUpdateInfo.endHash).to.eq(
                periodUpdate.update.finalizedExeHeader.parentHash
            );
        });

        it("updateLightClient should be failed when previous exe block headers are not updated ", async function () {
            let lightNode = await deployFixture();
            let initialized = await lightNode.initialized();
            expect(initialized).true;

            let update = await lightNode.getUpdateBytes(periodUpdate.update)
            await lightNode.updateLightClient(update);

            let headers = await lightNode.getHeadersBytes(exedata.headers)
            await lightNode.updateBlockHeader(headers);
            await expect(
                lightNode.updateLightClient(update)
            ).to.be.revertedWith(
                "previous exe block headers should be updated before update light client"
            );
        });
    });

    describe("Update execution header", function () {
        it("updateBlockHeader ... ok ", async function () {
            let lightNode = await loadFixture(deployFixture);

            let update = await lightNode.getUpdateBytes(periodUpdate.update)
            await lightNode.updateLightClient(update);

            let headers = await lightNode.getHeadersBytes(exedata.headers)
            await lightNode.updateBlockHeader(headers);

            let exeHeaderUpdateInfo = await lightNode.exeHeaderUpdateInfo();
            expect(exeHeaderUpdateInfo.startNumber).to.eq(
                bootstrap.finalizedExeHeaderNumber.toNumber() + 1
            );
            expect(exeHeaderUpdateInfo.endNumber).to.eq(
                exedata.headers[0].number - 1
            );
            expect(exeHeaderUpdateInfo.endHash).to.eq(exedata.headers[0].parentHash);
        });
    });

    describe("Verify proof data", function () {
        it("verifyProofData ... ok ", async function () {
            let lightNode = await loadFixture(deployFixture);

            let update = await lightNode.getUpdateBytes(periodUpdate.update)
            await lightNode.updateLightClient(update);
            let headers = await lightNode.getHeadersBytes(exedata.headers)
            await lightNode.updateBlockHeader(headers);

            let exeHeaderUpdateInfo = await lightNode.exeHeaderUpdateInfo();
            expect(exeHeaderUpdateInfo.startNumber).to.eq(
                bootstrap.finalizedExeHeaderNumber.toNumber() + 1
            );
            expect(exeHeaderUpdateInfo.endNumber).to.eq(
                exedata.headers[0].number - 1
            );

            let proofData = await lightNode.getBytes(exedata.receiptProof);
            let result = await lightNode.verifyProofData(proofData, {
                gasLimit: 20000000,
            });
            expect(result.success).to.true;
        });

        describe("Verifiable header range", function () {
            it("verifiableHeaderRange ... ok ", async function () {
                let lightNode = await loadFixture(deployFixture);

                let begin = await lightNode.verifiableHeaderRange();
                expect(begin[0]).to.eq(bootstrap.finalizedExeHeaderNumber);
                expect(begin[1]).to.eq(bootstrap.finalizedExeHeaderNumber);

                let update = await lightNode.getUpdateBytes(periodUpdate.update)
                await lightNode.updateLightClient(update);
                begin = await lightNode.verifiableHeaderRange();
                expect(begin[0]).to.eq(bootstrap.finalizedExeHeaderNumber);
                expect(begin[1]).to.eq(bootstrap.finalizedExeHeaderNumber);

                let headers = await lightNode.getHeadersBytes(exedata.headers)
                await lightNode.updateBlockHeader(headers);
                begin = await lightNode.verifiableHeaderRange();
                expect(begin[0]).to.eq(bootstrap.finalizedExeHeaderNumber);
                expect(begin[1]).to.eq(bootstrap.finalizedExeHeaderNumber);
            });
        });
    });
});

describe("LightNode Test on MAP", function () {
    let proxy: Contract;

    if (
        config.defaultNetwork != "local" &&
        config.defaultNetwork != "makalu" &&
        config.defaultNetwork != "dev"
    ) {
        return;
    }

    let verifyUpdate = true;
    console.log("verifyUpdate", verifyUpdate);

    describe("initialize and update", function () {
        it("initialization should be OK", async function () {
            let [wallet] = await ethers.getSigners();
            const MPTVerify = await ethers.getContractFactory("MPTVerify");
            const mPTVerify = await MPTVerify.deploy();
            await mPTVerify.connect(wallet).deployed();
            const LightNode = await ethers.getContractFactory("LightNode");
            const lightNode = await LightNode.deploy();
            await lightNode.connect(wallet).deployed();
            const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");

            let initData = LightNode.interface.encodeFunctionData("initialize", [
                chainId,
                wallet.address,
                mPTVerify.address,
                bootstrap.finalizedBeaconHeader,
                bootstrap.finalizedExeHeaderNumber,
                bootstrap.finalizedExeHeaderHash,
                bootstrap.curSyncCommitteAggPubkey,
                bootstrap.nextSyncCommitteAggPubkey,
                bootstrap.syncCommitteePubkeyHashes,
                verifyUpdate,
            ]);

            const lightNodeProxy = await LightNodeProxy.deploy(
                lightNode.address,
                initData
            );
            await lightNodeProxy.connect(wallet).deployed();
            proxy = LightNode.attach(lightNodeProxy.address);
            let initStage = await proxy.initStage();
            expect(initStage).to.eq(1);
            let initialized = await proxy.initialized();
            expect(initialized).false;
        });

        it("init cur sync committee pub keys should be OK", async function () {
            for (let i = 0; i < bootstrap.curSyncCommitteePubkeys.length; i++) {
                await proxy.initSyncCommitteePubkey(
                    bootstrap.curSyncCommitteePubkeys[i]
                );
                await delay(10000);
                let initStage = await proxy.initStage();
                expect(initStage).to.eq(2 + i);
                let initialized = await proxy.initialized();
                expect(initialized).false;
            }
        });

        it("init next sync committee pub keys should be OK", async function () {
            for (let i = 0; i < bootstrap.nextSyncCommitteePubkeys.length; i++) {
                await proxy.initSyncCommitteePubkey(
                    bootstrap.nextSyncCommitteePubkeys[i]
                );
                await delay(10000);
                let initStage = await proxy.initStage();
                let initialized = await proxy.initialized();
                if (i != bootstrap.nextSyncCommitteePubkeys.length - 1) {
                    expect(initStage).to.eq(5 + i);
                    expect(initialized).false;
                } else {
                    expect(initStage).to.eq(5 + i - 1);
                    expect(initialized).true;
                }
            }
        });

        it("updateLightClient should be OK ", async function () {
            let update = await proxy.getUpdateBytes(periodUpdate.update)
            await proxy.updateLightClient(update);
            await delay(10000);

            let finalizedSlot = await proxy.headerHeight();
            expect(finalizedSlot).to.eq(periodUpdate.update.finalizedHeader.slot);

            let exeHeaderUpdateInfo = await proxy.exeHeaderUpdateInfo();
            expect(exeHeaderUpdateInfo.startNumber).to.eq(
                bootstrap.finalizedExeHeaderNumber.toNumber() + 1
            );
            expect(exeHeaderUpdateInfo.endNumber).to.eq(
                periodUpdate.update.finalizedExeHeader.number - 1
            );
            expect(exeHeaderUpdateInfo.endHash).to.eq(
                periodUpdate.update.finalizedExeHeader.parentHash
            );
        });
    });
});
