const { LIGHTNODE_SALT,DEPLOY_FACTORY} = process.env;
const {types} = require("hardhat/config");

task("lightClientDeploy",
    "deploy LightNode proxy and init",
    require("./lightClientDeploy")
)
    .addOptionalParam("epoch", "init epoch default current epoch",0,types.int)
    .addOptionalParam("mpt", " Provable address","0xf0EEbaE3e4541b7762442a70046564be5330fA7D",types.string)
    .addOptionalParam("ledger", " LedgerInfo address","0x871264fb8A6F4584Fea4038D23f2f7f7B8166e3A",types.string)
    .addOptionalParam("salt", "deploy contract salt",LIGHTNODE_SALT , types.string)
    .addOptionalParam("factory", "mos contract address",DEPLOY_FACTORY , types.string)
