const { Conflux } = require("js-conflux-sdk");

module.exports = async (taskArgs, hre) => {
    const { deploy } = hre.deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    let conflux = new Conflux({
        url: "https://test.confluxrpc.com",
        networkId: 71,
    });

    let deployChainId = hre.network.config.chainId;
    if (deployChainId === 22776) {
        conflux = new Conflux({
            url: "https://main.confluxrpc.com",
            networkId: 1029,
        });
        console.log("deploy id :", deployChainId);
    }

    await deploy("LightNode", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "LightNode",
    });

    let lightNode = await deployments.get("LightNode");
    let LightNode = await ethers.getContractFactory("LightNode");
    console.log("light node implementation address:", lightNode.address);

    let epoch = await conflux.pos.getCommittee();
    let epochNumber;
    if (taskArgs.epoch === 0) {
        epochNumber = Number(epoch.currentCommittee.epochNumber) - 1;
        console.log(epochNumber);
    } else {
        epochNumber = taskArgs.epoch;
    }

    let preLedgerInfo = await conflux.provider.request({
        method: "pos_getLedgerInfoByEpoch",
        params: ["0x" + (Number(epochNumber) - 1).toString(16)],
    });

    let ledgerInfo = await conflux.provider.request({
        method: "pos_getLedgerInfoByEpochAndRound",
        params: ["0x" + epochNumber.toString(16), "0x1"],
    });

    let nextEpochValidators = [];
    let complementData = "00000000000000000000000000000000";
    for (let k in preLedgerInfo.nextEpochValidators) {
        let compressedPublicKey = preLedgerInfo.nextEpochValidators[k];
        console.log(preLedgerInfo.nextEpochValidators[k]);
        let uncompressedPublicKeyIndex =
            compressedPublicKey.substring(0, 98) + complementData + compressedPublicKey.substring(98);
        uncompressedPublicKeyIndex =
            uncompressedPublicKeyIndex.substring(0, 2) + complementData + uncompressedPublicKeyIndex.substring(2);
        let nextEpochValidator = {
            user: k,
            uncompressedPublicKey: uncompressedPublicKeyIndex,
        };
        nextEpochValidators.push(nextEpochValidator);
    }

    let validator = preLedgerInfo.ledgerInfo.commitInfo.nextEpochState.verifier.addressToValidatorInfo;
    console.log(validator);
    let chaosValidators = [];
    for (let i in validator) {
        chaosValidators.push(i);
    }

    chaosValidators = chaosValidators.sort();

    let validators = [];
    for (let i = 0; i < chaosValidators.length; i++) {
        let validatorInfo = {
            account: chaosValidators[i],
            uncompressedPublicKey: "",
            vrfPublicKey: validator[chaosValidators[i]].vrfPublicKey,
            votingPower: validator[chaosValidators[i]].votingPower,
        };
        validators.push(validatorInfo);
    }

    for (let h = 0; h < nextEpochValidators.length; h++) {
        if (validators[h].account == nextEpochValidators[h].user) {
            validators[h].uncompressedPublicKey = nextEpochValidators[h].uncompressedPublicKey;
        }
    }

    console.log(validators);

    let nextEpochStates = {
        epoch: preLedgerInfo.ledgerInfo.commitInfo.nextEpochState.epoch,
        validators: validators,
        quorumVotingPower: preLedgerInfo.ledgerInfo.commitInfo.nextEpochState.verifier.quorumVotingPower,
        totalVotingPower: preLedgerInfo.ledgerInfo.commitInfo.nextEpochState.verifier.totalVotingPower,
        vrfSeed: preLedgerInfo.ledgerInfo.commitInfo.nextEpochState.vrfSeed,
    };

    let accountSignature = [];

    for (let j in ledgerInfo.signatures) {
        accountSignature.push(j);
    }

    accountSignature = accountSignature.sort();

    let ledgerInfoSignatures = {
        epoch: ledgerInfo.ledgerInfo.commitInfo.epoch,
        round: ledgerInfo.ledgerInfo.commitInfo.round,
        id: ledgerInfo.ledgerInfo.commitInfo.id,
        executedStateId: ledgerInfo.ledgerInfo.commitInfo.executedStateId,
        version: ledgerInfo.ledgerInfo.commitInfo.version,
        timestampUsecs: ledgerInfo.ledgerInfo.commitInfo.timestampUsecs,
        nextEpochState: nextEpochStates,
        pivot: ledgerInfo.ledgerInfo.commitInfo.pivot,
        consensusDataHash: ledgerInfo.ledgerInfo.consensusDataHash,
        accounts: accountSignature,
        aggregatedSignature: ledgerInfo.aggregatedSignature,
    };

    console.log(ledgerInfoSignatures);

    let data = LightNode.interface.encodeFunctionData("initialize", [
        deployer.address,
        taskArgs.ledger,
        taskArgs.mpt,
        nextEpochStates,
        ledgerInfoSignatures,
    ]);

    let lightProxy = await ethers.getContractFactory("LightNodeProxy");

    let initData = await ethers.utils.defaultAbiCoder.encode(["address", "bytes"], [lightNode.address, data]);

    let deployData = lightProxy.bytecode + initData.substring(2);

    console.log("light node salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let factory = await ethers.getContractAt("IDeployFactory", taskArgs.factory);

    console.log("deploy factory address:", factory.address);

    await (await factory.connect(deployer).deploy(hash, deployData, 0, { gasLimit: 10000000 })).wait();

    let lightProxyAddress = await factory.connect(deployer).getAddress(hash);

    console.log("deployed light node proxy address:", lightProxyAddress);

    let proxy = await ethers.getContractAt("LightNode", lightProxyAddress);

    let owner = await proxy.connect(deployer).getAdmin();

    console.log(
        `LightNode Proxy contract address is ${lightProxyAddress}, init admin address is ${owner}, deploy contract salt is ${hash}`
    );
};
