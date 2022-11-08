
task("tokenDeploy",
    "Deploy a token with role control",
    require("./tokenDeploy")
)
    .addParam("name", "token name")
    .addParam("symbol", "token symbol")
    .addParam("balance", "init balance")

task("tokenGrant",
    "Grant a mintable token mint role",
    require("./tokenGrant")
)
    .addParam("token", "token address")
    .addParam("minter", "address/relay/mos, grant address can be an address or relay/mos")

task("mosDeploy",
    "Deploy the upgradeable MapCrossChainService contract and initialize it",
    require("./mosDeploy")
)
    .addParam("wrapped", "native wrapped token address")
    .addParam("lightnode", "lightNode contract address")

task("relayDeploy",
    "Deploy the upgradeable MapCrossChainServiceRelay contract and initialize it",
    require("./relayDeploy")
)
    .addParam("wrapped", "native wrapped token address")
    .addParam("lightnode", "lightNodeManager contract address")

task("mosSetRelay",
    "Initialize MapCrossChainServiceRelay address for MapCrossChainService",
    require("./mosSetRelay")
)
    .addParam("relay", "map chain relay contract address")
    .addParam("chain", "map chain id")

task("mosSetBridgeToken",
    "MapCrossChainService settings allow cross-chain tokens",
    require("./mosSetBridgeToken")
)
    .addParam("token", "token address")
    .addParam("chains", "The chain id that is allowed to cross can be filled with multiple ',' separated by example (1,2,3)")

task("mosSetMintableToken",
    "MapCrossChainService settings mintable token",
    require("./mosSetMintableToken")
)
    .addParam("token", "token address")
    .addParam("mintable", "true or false")

task("relayInit",
    "Initialize MapCrossChainServiceRelay contract",
    require("./relayInit")
)
    .addParam("feecenter","feeCenter contract")
    .addParam("tokenregister","tokenRegister contract")

task("relaySetMintableToken",
    "MapCrossChainServiceRelay settings mintable token",
    require("./relaySetMintableToken")
)
    .addParam("token", "token address")
    .addParam("mintable", "true or false")


task("relayRegisterChain",
    "Register altchain MapCrossChainService to MapCrossChainServiceRelay",
    require("./relayRegisterChain")
)
    .addParam("address", "MapCrossChainService contract address")
    .addParam("chain", "MapCrossChainService chain id")

task("relayInitNear",
    "MapCrossChainServiceRelay initializes near chain settings",
    require("./relayInitNear")
)
    .addParam("chain", "chain id")

task("relaySetTokenDecimals",
    "Set the decimals of maptoken corresponding to other chains",
    require("./relaySetTokenDecimals")
)
    .addParam("token", "Token address")
    .addParam("chain", "Cross-chain chainId")
    .addParam("decimals", "Token decimals")

task("relaySetVaultBalance",
    "MapCrossChainServiceRelay sets cross-chain token quota",
    require("./relaySetVaultBalance")
)
    .addParam("chain", "Chain id that allows cross-chain")
    .addParam("token", "Token address")
    .addParam("balance", "Allowed Amount")

task("feeBindTokenVault",
    "Binding fee address to provide liquidity vault address",
    require("./feeBindTokenVault")
)
    .addParam("vault", "vault address")
    .addParam("token", "The maptoken address corresponding to the cross-chain token")

task("feeSetDistributeRate",
    "Set the fee distribution rate, when getting fee from every crosschain transaction, the fee will be distributed to specified address",
    require("./feeSetDistributeRate")
)
    .addParam("type", "0 or 1, type 0 is vault")
    .addParam("address", "vault address")
    .addParam("rate", "The percentage value of the fee charged, 0-1000000, unit is 0.000001")

task("feeSetTokenFee",
    "Set token fee to target chain",
    require("./feeSetTokenFee")
)
    .addParam("token", "relay chain token address")
    .addParam("chain", "target chain id")
    .addParam("min", "One-time cross-chain charging minimum handling fee")
    .addParam("max", "One-time cross-chain charging maximum handling fee")
    .addParam("rate", "The percentage value of the fee charged")

task("registerToken",
    "Mapping settings for tokens that require cross-chain between two chains",
    require("./registerToken")
)
    .addParam("token", "relay chain token address")
    .addParam("chain", "cross-chain id")
    .addParam("chaintoken", "cross-chain token")


task("vaultInitToken",
    "Initialize the vaultToken",
    require("./vaultInitToken")
)
    .addParam("token", "The token address mapped by the cross-chain token on the map chain")
    .addParam("name", "The name of the vault token")
    .addParam("symbol", "The symbol of the vault token")

task("transfer",
    "Cross-chain transfer token",
    require("./transfer")
)
    .addParam("token", "The token address")
    .addParam("address", "The receiver address")
    .addParam("value", "transfer value, unit WEI")
    .addParam("chain", "target chain id")