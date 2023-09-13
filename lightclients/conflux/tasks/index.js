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
