
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
    .addParam("feeCenter","feeCenter contract")
    .addParam("tokenRegister","tokenRegister contract")


task("SetCanBridgeToken",
    "MapCrossChainService settings allow cross-chain tokens",
    require("./mapCrossChainServiceSetCanBridgeToken")
)
    .addParam("token", "token address")
    .addParam("chains", "The chain id that is allowed to cross can be filled with multiple ',' separated by example (1,2,3)")


task("registerMCS",
    "Register AltChain MapCrossChainService to MapCrossChainServiceRelay",
    require("./mapCrossChainServiceRelaySetBridgeAddress")
)
    .addParam("address", "MapCrossChainService contract address")
    .addParam("chain", "The chain id where MapCrossChainService is located")


task("mcsSetChain",
    "MapCrossChainService initializes near chain settings",
    require("./mapCrossChainServiceSetChain")
)
    .addParam("chain", "chain id")
    .addParam("name", "chain name")

task("mapCrossChainServiceRelaySetChain",
    "MapCrossChainServiceRelay initializes near chain settings",
    require("./mapCrossChainServiceRelaySetChain")
)
    .addParam("chain", "chain id")
    .addParam("name","chain name")

task("mapCrossChainServiceRelaySetTokenDecimals",
    "Set the decimals of maptoken corresponding to other chains",
    require("./tokenRegisterSetTokenDecimals")
)
    .addParam("token", "Token address")
    .addParam("chains", "Cross-chain chainId")
    .addParam("decimals", "Token decimals")

task("mcsRelaySetVaultBalance",
    "MapCrossChainServiceRelay sets cross-chain token quota",
    require("./mapCrossChainServiceRelaySetVaultBalance")
)
    .addParam("chain", "Chain id that allows cross-chain")
    .addParam("token", "Token address")
    .addParam("balance", "Allowed Amount")

task("feeCenterSetTokenVault",
    "Binding fee address to provide liquidity vault address",
    require("./feeCenterSetTokenVault")
)
    .addParam("vault", "vault address")
    .addParam("token", "The maptoken address corresponding to the cross-chain token")

task("feeCenterSetDistributeRate",
    "Set the fee to enter the vault address",
    require("./feeCenterSetDistributeRate")
)
    .addParam("token", "vault address")
    .addParam("rate", "The percentage value of the fee charged")

task("feeCenterSetChainTokenGasFee",
    "Set fees for tokens",
    require("./feeCenterSetChainTokenGasFee")
)
    .addParam("chain", "Allow cross-chain id")
    .addParam("token", "token address")
    .addParam("min", "One-time cross-chain charging minimum handling fee")
    .addParam("max", "One-time cross-chain charging maximum handling fee")
    .addParam("rate", "The percentage value of the fee charged")

task("tokenRegisterRegToken",
    "Mapping settings for tokens that require cross-chain between two chains",
    require("./tokenRegisterRegToken")
)
    .addParam("chain", "cross-chain id")
    .addParam("token", "cross-chain token")
    .addParam("mapToken", "Map token corresponding to map chain")

task("vaultTokenInit",
    "Initialize the vaultToken",
    require("./vaultTokenInit")
)
    .addParam("token", "The token address mapped by the cross-chain token on the map chain")
    .addParam("name", "The name of the vault token")
    .addParam("symbol", "The symbol of the vault token")