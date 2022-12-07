
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


task("vaultDeploy",
    "Deploy the vault token",
    require("./vaultDeploy")
)
    .addParam("token", "The token address on relay chain")
    .addParam("name", "The name of the vault token")
    .addParam("symbol", "The symbol of the vault token")

task("vaultAddManager",
    "Add vaultToken manager",
    require("./vaultAddManager")
)
    .addParam("vault", "The vault token address")
    .addOptionalParam("manager", "the manager address, default is relay", "relay", types.string)

//Organized

task("tokenDeploy",
    "Deploy a token with role control",
    require("./tokenDeploy")
)
    .addParam("name", "token name")
    .addParam("symbol", "token symbol")
    .addOptionalParam("decimals", "default 18", 18, types.int)
    .addOptionalParam("balance", "init balance, default 0", 0, types.int)

task("tokenGrant",
    "Grant a mintable token mint role",
    require("./tokenGrant")
)
    .addParam("token", "token address")
    .addOptionalParam("minter", "minter address, default mos", "mos", types.string)

task("tokenMint",
    "mint token",
    require("./tokenMint")
)
    .addParam("token", "token address")
    .addParam("amount", "mint amount")

task("mosSetRelay",
    "Initialize MapCrossChainServiceRelay address for MapCrossChainService",
    require("./mosSetRelay")
)
    .addParam("relay", "map chain relay contract address")
    .addParam("chain", "map chain id")

task("mosSetClient",
    "Set light client address for MapCrossChainService",
    require("./mosSetClient")
)
    .addParam("client", "light client address")

task("mosRegisterToken",
    "MapCrossChainService settings allow cross-chain tokens",
    require("./mosRegisterToken")
)
    .addParam("token", "token address")
    .addParam("chains", "chain ids allowed to cross, separated by ',', ex. `1,2,3` ")
    .addOptionalParam("enable", "true or false", true, types.boolean)

task("mosSetMintableToken",
    "MapCrossChainService settings mintable token",
    require("./mosSetMintableToken")
)
    .addParam("token", "token address")
    .addParam("mintable", "true or false",false,types.boolean)

task("relayInit",
    "Initialize mos contract",
    require("./relayInit")
)
    .addParam("tokenmanager","tokenRegister contract")

task("relaySetClientManager",
    "Update client manager",
    require("./relaySetClientManager")
)
    .addParam("manager","client manager contract")

task("relayRegisterChain",
    "Register altchain mos to relay chain",
    require("./relayRegisterChain")
)
    .addParam("address", "mos contract address")
    .addParam("chain", "chain id")
    .addOptionalParam("type", "chain type, default 1", 1, types.int)


task("relayRegisterToken",
    "Register cross-chain token on relay chain",
    require("./relayRegisterToken")
)
    .addParam("token", "Token address")
    .addParam("vault", "vault token address")
    .addParam("mintable", "token mintable",false,types.boolean)




task("relayMapToken",
    "Map the altchain token to the token on relay chain",
    require("./relayMapToken")
)
    .addParam("token", "token address to relay chain")
    .addParam("chain", "cross-chain id")
    .addParam("chaintoken", "cross-chain token")
    .addOptionalParam("decimals", "token decimals, default 18", 18, types.int)


task("relaySetTokenFee",
    "Set token fee to target chain",
    require("./relaySetTokenFee")
)
    .addParam("token", "relay chain token address")
    .addParam("chain", "target chain id")
    .addParam("min", "One-time cross-chain charging minimum handling fee")
    .addParam("max", "One-time cross-chain charging maximum handling fee")
    .addParam("rate", "The percentage value of the fee charged, unit is 0.000001")

task("relaySetDistributeRate",
    "Set the fee to enter the vault address",
    require("./relaySetDistributeRate")
)
    .addOptionalParam("type", "0 or 1, type 0 is vault, default 0", 0, types.int)
    .addOptionalParam("address", "receiver address", "0x0000000000000000000000000000000000000DEF", types.string)
    .addParam("rate", "The percentage value of the fee charged, unit 0.000001")


task("transferOutToken",
    "Cross-chain transfer token",
    require("./transferOutToken")
)
    .addParam("mos", "the mos address")
    .addOptionalParam("token", "The token address","0x0000000000000000000000000000000000000000",types.string)
    .addOptionalParam("address", "The receiver address, default is msg.sender","",types.string)
    .addParam("value", "transfer value, unit WEI")
    .addParam("chain", "target chain id")

task("depositOutToken",
    "Cross-chain deposit token",
    require("./depositOutToken")
)
    .addParam("mos", "The mos address")
    .addOptionalParam("token", "The token address","0x0000000000000000000000000000000000000000",types.string)
    .addOptionalParam("address", "The receiver address","",types.string)
    .addParam("value", "deposit value, unit WEI")

task("withdraw",
    "withdraw token",
    require("./withdraw")
)
    .addParam("mos", "The mos address")
    .addOptionalParam("token", "The token address","0x0000000000000000000000000000000000000000",types.string)
    .addOptionalParam("address", "The receiver address","",types.string)
    .addParam("value", "withdraw value")

task("relayList",
    "List mos relay infos",
    require("./relayList")
)
    .addOptionalParam("relay", "The mos address, default mos", "relay", types.string)
    .addOptionalParam("token", "The token address, default wtoken", "wtoken", types.string)

task("mosList",
    "List mos relay infos",
    require("./mosList")
)
    .addOptionalParam("mos", "The mos address, default mos", "mos", types.string)
    .addOptionalParam("token", "The token address, default wtoken", "wtoken", types.string)