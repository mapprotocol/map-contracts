import {HardhatRuntimeEnvironment} from 'hardhat/types';
import {DeployFunction} from 'hardhat-deploy/types';
import {dynamicImport} from 'tsimportlib';
import {LightClientUpdate} from "@lodestar/types/lib/altair/types";
import {bellatrix} from "@lodestar/types";
import {hexlify} from "ethers/lib/utils";
import {expect} from "chai";
import {computePubkeyHash, delay, getPubkeySlice} from "../utils/Util";

const url = process.env.URL!;
let chainId = process.env.CHAIN_ID;
let blockRoot = process.env.TRUSTED_BLOCK_ROOT!;

const deploy: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
    const {deployments, getNamedAccounts, ethers} = hre;
    const {deploy} = deployments;
    const {getClient} = await dynamicImport('@lodestar/api', module) as typeof import('@lodestar/api');
    const {config} = await dynamicImport('@lodestar/config/default', module) as typeof import('@lodestar/config/default');

    let [wallet] = await ethers.getSigners();
    const {deployer} = await getNamedAccounts();

    let mPTVerify = await deployments.get('MPTVerify');
    let lightNode = await deployments.get('LightNode');

    let LightNode = await ethers.getContractFactory("LightNode")

    const api = getClient({baseUrl: url}, {config});

    let boorStrapResp = await api.lightclient.getBootstrap(blockRoot)
    let bootStrap = boorStrapResp.data
    let period = getPeriodBySlot(bootStrap.header.slot)
    console.log("period", period)
    let periodUpdateResp = await api.lightclient.getUpdates(period, 1)
    let periodUpdate: LightClientUpdate = periodUpdateResp.data[0];
    let finalizedBlock = await api.beacon.getBlockV2(bootStrap.header.slot);
    let block: bellatrix.SignedBeaconBlock = finalizedBlock.data;
    let finalizedExeHeaderNumber = block.message.body.executionPayload.blockNumber;
    let finalizedExeHeaderHash = block.message.body.executionPayload.blockHash;

    console.log("finalizedHeader", bootStrap.header)
    console.log("finalizedExeHeaderNumber", hexlify(finalizedExeHeaderNumber))
    console.log("finalizedExeHeaderHash", hexlify(finalizedExeHeaderHash))
    console.log("curSyncCommittee.aggregatePubkey", hexlify(bootStrap.currentSyncCommittee.aggregatePubkey))
    console.log("nextSyncCommittee.aggregatePubkey", hexlify(periodUpdate.nextSyncCommittee.aggregatePubkey))


    console.log("period finalizedHeader slot", periodUpdate.finalizedHeader.slot)

    let hashes: string[] = [];
    hashes.push(computePubkeyHash(bootStrap.currentSyncCommittee.pubkeys, 0, 171))
    hashes.push(computePubkeyHash(bootStrap.currentSyncCommittee.pubkeys, 171, 342))
    hashes.push(computePubkeyHash(bootStrap.currentSyncCommittee.pubkeys, 342, 512))
    hashes.push(computePubkeyHash(periodUpdate.nextSyncCommittee.pubkeys, 0, 171))
    hashes.push(computePubkeyHash(periodUpdate.nextSyncCommittee.pubkeys, 171, 342))
    hashes.push(computePubkeyHash(periodUpdate.nextSyncCommittee.pubkeys, 342, 512))
    console.log("hashes", hashes)

    let initData = LightNode.interface.encodeFunctionData(
        "initialize",
        [chainId,
            wallet.address,
            mPTVerify.address,
            bootStrap.header,
            finalizedExeHeaderNumber,
            finalizedExeHeaderHash,
            bootStrap.currentSyncCommittee.aggregatePubkey,
            periodUpdate.nextSyncCommittee.aggregatePubkey,
            hashes,
            false
        ]
    );


    const lightNodeProxy = await deploy('LightNodeProxy', {
        from: deployer,
        args: [lightNode.address, initData],
        log: true,
        contract: 'LightNodeProxy',
        gasLimit: 20000000
    });

    let proxy = LightNode.attach(lightNodeProxy.address);
    let initialized = await proxy.initialized();
    expect(initialized).false;
    let initStage = await proxy.initStage();
    expect(initStage).to.eq(1);

    console.log("init cur sync committee pubkeys part 1...")
    console.log(hexlify(getPubkeySlice(bootStrap.currentSyncCommittee.pubkeys, 0, 171)))
    await proxy.initSyncCommitteePubkey(getPubkeySlice(bootStrap.currentSyncCommittee.pubkeys, 0, 171));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(2);

    console.log("init cur sync committee pubkeys part 2...")
    console.log(hexlify(getPubkeySlice(bootStrap.currentSyncCommittee.pubkeys, 171, 342)))
    await proxy.initSyncCommitteePubkey(getPubkeySlice(bootStrap.currentSyncCommittee.pubkeys, 171, 342));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(3);

    console.log("init cur sync committee pubkeys part 3...")
    console.log(hexlify(getPubkeySlice(bootStrap.currentSyncCommittee.pubkeys, 342, 512)))
    await proxy.initSyncCommitteePubkey(getPubkeySlice(bootStrap.currentSyncCommittee.pubkeys, 342, 512));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(4);

    console.log("init next sync committee pubkeys part 1...")
    console.log(hexlify(getPubkeySlice(periodUpdate.nextSyncCommittee.pubkeys, 0, 171)))
    await proxy.initSyncCommitteePubkey(getPubkeySlice(periodUpdate.nextSyncCommittee.pubkeys, 0, 171));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(5);

    console.log("init next sync committee pubkeys part 2...")
    console.log(hexlify(getPubkeySlice(periodUpdate.nextSyncCommittee.pubkeys, 171, 342)))
    await proxy.initSyncCommitteePubkey(getPubkeySlice(periodUpdate.nextSyncCommittee.pubkeys, 171, 342));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(6);

    console.log("init next sync committee pubkeys part 3...")
    console.log(hexlify(getPubkeySlice(periodUpdate.nextSyncCommittee.pubkeys, 342, 512)))
    await proxy.initSyncCommitteePubkey(getPubkeySlice(periodUpdate.nextSyncCommittee.pubkeys, 342, 512));
    await delay(10000)

    initialized = await proxy.initialized();
    expect(initialized).true;
};

function getPeriodBySlot(slot : number) : number {
    return ~~(slot / 32 / 256)
}

export default deploy;
deploy.tags = ['Proxy'];
deploy.dependencies = ['LightNode']