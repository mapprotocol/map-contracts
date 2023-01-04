import {HardhatRuntimeEnvironment} from 'hardhat/types';
import {DeployFunction} from 'hardhat-deploy/types';
import {dynamicImport} from 'tsimportlib';
import {LightClientUpdate} from "@lodestar/types/lib/altair/types";
import {bellatrix} from "@lodestar/types";
import {expect} from "chai";
import {computePubkeyHash, delay, getPubkeySlice} from "../utils/Util";

const url = process.env.URL!;
let chainId = process.env.CHAIN_ID;
let period = parseInt(process.env.PERIOD!);

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

    console.log("period", period)
    let periodUpdate = await api.lightclient.getUpdates(period! - 1, 2)
    let prePeriod: LightClientUpdate = periodUpdate.data[0];
    let initPeriod: LightClientUpdate = periodUpdate.data[1];

    console.log("initPeriod.finalizedHeader.slot", initPeriod.finalizedHeader.slot)

    let finalizedBlock = await api.beacon.getBlockV2(initPeriod.finalizedHeader.slot);
    let block: bellatrix.SignedBeaconBlock = finalizedBlock.data;
    let finalizedExeHeaderNumber = block.message.body.executionPayload.blockNumber;
    let finalizedExeHeaderHash = block.message.body.executionPayload.blockHash;

    console.log("initPeriod.finalizedHeader", initPeriod.finalizedHeader)
    console.log("finalizedExeHeaderNumber", finalizedExeHeaderNumber)
    console.log("finalizedExeHeaderHash", finalizedExeHeaderHash)
    console.log("prePeriod.nextSyncCommittee.aggregatePubkey", prePeriod.nextSyncCommittee.aggregatePubkey)
    console.log("initPeriod.nextSyncCommittee.aggregatePubkey", initPeriod.nextSyncCommittee.aggregatePubkey)

    let hashes: string[] = [];
    hashes.push(computePubkeyHash(prePeriod.nextSyncCommittee.pubkeys, 0, 171))
    hashes.push(computePubkeyHash(prePeriod.nextSyncCommittee.pubkeys, 171, 342))
    hashes.push(computePubkeyHash(prePeriod.nextSyncCommittee.pubkeys, 342, 512))
    hashes.push(computePubkeyHash(initPeriod.nextSyncCommittee.pubkeys, 0, 171))
    hashes.push(computePubkeyHash(initPeriod.nextSyncCommittee.pubkeys, 171, 342))
    hashes.push(computePubkeyHash(initPeriod.nextSyncCommittee.pubkeys, 342, 512))
    console.log("hashes", hashes)

    let initData = LightNode.interface.encodeFunctionData(
        "initialize",
        [chainId,
            wallet.address,
            mPTVerify.address,
            initPeriod.finalizedHeader,
            finalizedExeHeaderNumber,
            finalizedExeHeaderHash,
            prePeriod.nextSyncCommittee.aggregatePubkey,
            initPeriod.nextSyncCommittee.aggregatePubkey,
            hashes,
            true
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
    await proxy.initSyncCommitteePubkey(getPubkeySlice(prePeriod.nextSyncCommittee.pubkeys, 0, 171));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(2);

    console.log("init cur sync committee pubkeys part 2...")
    await proxy.initSyncCommitteePubkey(getPubkeySlice(prePeriod.nextSyncCommittee.pubkeys, 171, 342));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(3);

    console.log("init cur sync committee pubkeys part 3...")
    await proxy.initSyncCommitteePubkey(getPubkeySlice(prePeriod.nextSyncCommittee.pubkeys, 342, 512));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(4);

    console.log("init next sync committee pubkeys part 1...")
    await proxy.initSyncCommitteePubkey(getPubkeySlice(initPeriod.nextSyncCommittee.pubkeys, 0, 171));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(5);

    console.log("init next sync committee pubkeys part 2...")
    await proxy.initSyncCommitteePubkey(getPubkeySlice(initPeriod.nextSyncCommittee.pubkeys, 171, 342));
    await delay(10000)
    initStage = await proxy.initStage();
    expect(initStage).to.eq(6);

    console.log("init next sync committee pubkeys part 3...")
    await proxy.initSyncCommitteePubkey(getPubkeySlice(initPeriod.nextSyncCommittee.pubkeys, 342, 512));
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