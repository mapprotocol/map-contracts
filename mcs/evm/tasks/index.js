
task("deployToken",
    "Deploy a token with role control",
    require("./deployToken")
)
    .addParam("name", "token name")
    .addParam("symbol", "token symbol")
    .addParam("balance", "init balance")

task("grantToken",
    "Grant a mintable token mint role",
    require("./grantToken")
)
    .addParam("token", "token address")
    .addParam("minter", "address/relay/mos, grant address can be an address or relay/mos")

task("deployMCS",
    "Deploy the upgradeable MapCrossChainService contract and initialize it",
    require("./deployMapCrossChainServiceProxy")
)
    .addParam("wrapped", "native wrapped token address")
    .addParam("maptoken", "map token address")
    .addParam("lightnode", "lightNode contract address")

task("deployRelay",
    "Deploy the upgradeable MapCrossChainServiceRelay contract and initialize it",
    require("./deployMapCrossChainServiceRelayProxy")
)
    .addParam("wrapped", "native wrapped token address")
    .addParam("lightnode", "lightNodeManager contract address")

task("initMCS",
    "Initialize MapCrossChainServiceRelay address for MapCrossChainService",
    require("./mapCrossChainServiceSet")
)
    .addParam("relay", "map chain relay contract address")
    .addParam("chain", "map chain id")

task("initRelay",
    "Initialize MapCrossChainServiceRelay contract",
    require("./mapCrossChainServiceRelaySet")
)
    .addParam("feecenter", "fee center contract address")
    .addParam("register", "token register contract address")

task("mapCrossChainServiceSetCanBridgeToken",
    "MapCrossChainService settings allow cross-chain tokens",
    require("./mapCrossChainServiceSetCanBridgeToken")
)
    .addParam("tokenaddress", "token address")
    .addParam("ids", "The chain id that is allowed to cross can be filled with multiple ',' separated by example (1,2,3)")


task("registerMCS",
    "Register AltChain MapCrossChainService to MapCrossChainServiceRelay",
    require("./mapCrossChainServiceRelaySetBridgeAddress")
)
    .addParam("address", "MapCrossChainService contract address")
    .addParam("chain", "The chain id where MapCrossChainService is located")


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