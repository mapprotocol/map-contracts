const { LIGHTNODE_SALT,DEPLOY_FACTORY} = process.env;
const {types} = require("hardhat/config");

task("lightClientDeploy",
    "deploy LightNode proxy and init",
    require("./lightClientDeploy")
)
    .addParam("mpt", " Provable address")
    .addParam("ledger", " LedgerInfo address")
    .addOptionalParam("epoch", "Init epoch default current epoch sub 1",0,types.int)
    .addOptionalParam("salt", "Deploy contract salt",LIGHTNODE_SALT , types.string)
    .addOptionalParam("factory", "Deploy factory contract address",DEPLOY_FACTORY , types.string)


task("setLedgerInfo", "LightNode set the LedgerInfo contract address", require("./setLedgerInfo"))
    .addParam("ledger", "LedgerInfo contract address")

task("factorySetLedgerInfo", "LightNode set the LedgerInfo contract address", require("./factorySetLedgerInfo"))
    .addParam("ledger", "LedgerInfo contract address")
    .addOptionalParam("salt", "deploy contract salt", LIGHTNODE_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);


task("setProvable", "LightNode set the Provable contract address", require("./setProvable"))
    .addParam("provable", "Provable contract address")

task("factorySetProvable", "LightNode set the Provable contract address", require("./factorySetProvable"))
    .addParam("provable", "Provable contract address")
    .addOptionalParam("salt", "deploy contract salt", LIGHTNODE_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);