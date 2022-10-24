
task("deployCrossToken",
    "Deploy a token with role control",
    require("./deployCrossToken")
)
    .addParam("name", "token name")
    .addParam("symbol", "token symbol")

task("deployMapCrossChainServiceProxy",
    "Deploy the upgradeable MapCrossChainServiceProxy contract and initialize",
    require("./deployMapCrossChainServiceProxy")
)
    .addParam("weth", "weth token address")
    .addParam("maptoken", "map token address")
    .addParam("lightnode", "lightNode contract address")

task("deployMapCrossChainServiceRelayProxy",
    "Deploy the upgradeable MapCrossChainServiceRelayProxy contract and initialize",
    require("./deployMapCrossChainServiceRelayProxy")
)
    .addParam("weth", "weth token address")
    .addParam("maptoken", "map token address")
    .addParam("lightnode", "lightNodeManager contract address")

task("mapCrossChainServiceSet",
    "Do some basic settings for MapCrossChainService",
    require("./mapCrossChainServiceSet")
)
    .addParam("relayaddress", "map chain relay contract address")
    .addParam("chainid", "map chain id")

task("mapCrossChainServiceRelaySet",
    "Do some basic settings for MapCrossChainServiceRelay",
    require("./mapCrossChainServiceRelaySet")
)
    .addParam("feecenter", "feeCenter contract address")
    .addParam("registertoken", "registertoken contract address")

task("mapCrossChainServiceSetCanBridgeToken",
    "MapCrossChainService settings allow cross-chain tokens",
    require("./mapCrossChainServiceSetCanBridgeToken")
)
    .addParam("tokenaddress", "token address")
    .addParam("ids", "The chain id that is allowed to cross can be filled with multiple ',' separated by example (1,2,3)")


task("mapCrossChainServiceRelaySetBridgeAddress",
    "Receive cross-chain request addresses from other MapCrossChainService addresses",
    require("./mapCrossChainServiceRelaySetBridgeAddress")
)
    .addParam("mcsaddr", "MapCrossChainService contract address")
    .addParam("mcsid", "The id of the chain where MapCrossChainService is located")


task("mapCrossChainServiceInitNear",
    "MapCrossChainService initializes near chain settings",
    require("./mapCrossChainServiceInitNear")
)
    .addParam("nearid", "near chain id")

task("mapCrossChainServiceRelayInitNear",
    "MapCrossChainServiceRelay initializes near chain settings",
    require("./mapCrossChainServiceRelayInitNear")
)
    .addParam("nearid", "near chain id")

task("mapCrossChainServiceRelaySetTokenDecimals",
    "Set the decimals of maptoken corresponding to other chains",
    require("./mapCrossChainServiceRelaySetTokenDecimals")
)
    .addParam("tokenaddress", "Token address")
    .addParam("mcsids", "Cross-chain chainId")
    .addParam("tokendecimals", "Token decimals")

task("mapCrossChainServiceRelaySetVaultBalance",
    "MapCrossChainServiceRelay sets cross-chain token quota",
    require("./mapCrossChainServiceRelaySetVaultBalance")
)
    .addParam("mcsid", "Chain id that allows cross-chain")
    .addParam("tokenaddress", "Token address")
    .addParam("tokennumber", "Allowed Amount")

task("feeCenterSetTokenVault",
    "Binding fee address to provide liquidity vault address",
    require("./feeCenterSetTokenVault")
)
    .addParam("vaulttoken", "vault address")
    .addParam("crosstoken", "The maptoken address corresponding to the cross-chain token")

task("feeCenterSetDistributeRate",
    "Set the fee to enter the vault address",
    require("./feeCenterSetDistributeRate")
)
    .addParam("vaulttoken", "vault address")
    .addParam("ratenumber", "The percentage value of the fee charged")

task("feeCenterSetChainTokenGasFee",
    "Set fees for tokens",
    require("./feeCenterSetChainTokenGasFee")
)
    .addParam("mcschainid", "Allow cross-chain id")
    .addParam("crosstoken", "token address")
    .addParam("minfee", "One-time cross-chain charging minimum handling fee")
    .addParam("maxfee", "One-time cross-chain charging maximum handling fee")
    .addParam("ratefee", "The percentage value of the fee charged")

task("tokenRegister",
    "Mapping settings for tokens that require cross-chain between two chains",
    require("./tokenRegister")
)
    .addParam("crossid", "cross-chain id")
    .addParam("crosstoken", "cross-chain token")
    .addParam("maptoken", "Map token corresponding to map chain")

task("vaultTokenInit",
    "Initialize the vaultToken",
    require("./vaultTokenInit")
)
    .addParam("correspond", "The token address mapped by the cross-chain token on the map chain")
    .addParam("vaultname", "The name of the vault token")
    .addParam("vaultsymbol", "The symbol of the vault token")