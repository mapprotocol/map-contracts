const configData = require("./config/tokenCrossChainConfig.js");

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let vaultToken = await ethers.getContractAt('MAPVaultToken',configData.ralayVaultTokenAddress);
    let tokenRegister = await ethers.getContractAt('TokenRegister',configData.relayTokenRegisterAddress);
    let feeCenter = await ethers.getContractAt('FeeCenter',configData.relayFeeCenterAddress);
    let mcssRelayProxy = await ethers.getContractAt('MAPCrossChainServiceRelay',configData.relayAddress);

    await (await vaultToken.connect(deployer).initialize(
        configData.ralayMotTokenAddress,
        configData.relayVaultTokenName,
        configData.relayVaultTokenSymbol,
        "18")
    ).wait();
    console.log("MAPVaultToken initialize success")

    await (await feeCenter.connect(deployer).setTokenVault(
        configData.ralayMotTokenAddress,
        configData.ralayVaultTokenAddress
    )).wait();
    console.log("FeeCenter set setTokenVault success")

    await (await feeCenter.connect(deployer).setDistributeRate(
        "1",
        configData.ralayVaultTokenAddress,
        "1000"
    )).wait();

    console.log("FeeCenter set setDistributeRate success")

    await (await feeCenter.connect(deployer).setChainTokenGasFee(
            configData.mcsChainId,
            configData.ralayMotTokenAddress,
            configData.minFee,
            configData.maxFee,
            configData.rateFee)
    ).wait();

    console.log("FeeCenter set setChainTokenGasFee success")

    if (configData.nearChainId !== ""){
        await (await tokenRegister.connect(deployer).regToken(
            configData.mcsChainId,
            configData.mcsMotTokenAddress,
            configData.ralayMotTokenAddress
        )).wait()
        await (await tokenRegister.connect(deployer).regToken(
            configData.nearChainId,
            configData.nearMotTokenAddress,
            configData.ralayMotTokenAddress
        )).wait()
        await (await tokenRegister.connect(deployer).regToken(
            configData.relayChainId,
            configData.ralayMotTokenAddress,
            configData.mcsMotTokenAddress
        )).wait()
        console.log("TokenRegister set mcs near and relay token success")
    }else{
        await (await tokenRegister.connect(deployer).regToken(
            configData.mcsChainId,
            configData.mcsMotTokenAddress,
            configData.ralayMotTokenAddress
        )).wait()
        await (await tokenRegister.connect(deployer).regToken(
            configData.relayChainId,
            configData.ralayMotTokenAddress,
            configData.mcsMotTokenAddress
        )).wait()
        console.log("TokenRegister set mcs and relay token success")
    }

    if (configData.nearChainId !== ""){
        await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(
            configData.ralayMotTokenAddress,
            configData.mcsChainId,
            configData.mcsMotTokenDecimals
        )).wait()
        await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(
            configData.ralayMotTokenAddress,
            configData.relayChainId,
            configData.ralayMotTokenDecimals
        )).wait()
        await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(
            configData.ralayMotTokenAddress,
            configData.nearChainId,
            configData.nearMotTokenDecimals
        )).wait()

        console.log("MAPCrossChainServiceRelay set mcs relay near setTokenOtherChainDecimals success")

        await (await mcssRelayProxy.connect(deployer).setVaultBalance(
            configData.nearChainId,
            configData.ralayMotTokenAddress,
            configData.relayToMcsLimit
        )).wait()

        await (await mcssRelayProxy.connect(deployer).setVaultBalance(
            configData.mcsChainId,
            configData.ralayMotTokenAddress,
            configData.relayToNearLimit
        )).wait()

        console.log("MAPCrossChainServiceRelay set relay to mcs and near setVaultBalance success")
    }else{
        await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(
            configData.ralayMotTokenAddress,
            configData.mcsChainId,
            configData.mcsMotTokenDecimals
        )).wait()
        await (await mcssRelayProxy.connect(deployer).setTokenOtherChainDecimals(
            configData.ralayMotTokenAddress,
            configData.relayChainId,
            configData.ralayMotTokenDecimals
        )).wait()

        console.log("MAPCrossChainServiceRelay set mcs relay setTokenOtherChainDecimals success")

        await (await mcssRelayProxy.connect(deployer).setVaultBalance(
            configData.mcsChainId,
            configData.ralayMotTokenAddress,
            configData.relayToMcsLimit
        )).wait()
        console.log("MAPCrossChainServiceRelay set relay to mcs setVaultBalance success")
    }


}

module.exports.tags = ['TokenCrossChainSet'];