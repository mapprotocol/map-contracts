import {task, types} from "hardhat/config";
import * as dotenv from "dotenv";
dotenv.config();

const { LIGHTNODE_BTC_SALT,DEPLOY_FACTORY} = process.env;

task("lightFactoryDeploy",
    "deploy LightNode proxy and init",
    require("./lightFactoryDeploy")
)
    .addParam("height", "init height")
    .addParam("header", " block header")
    .addOptionalParam("salt", "deploy contract salt",LIGHTNODE_BTC_SALT , types.string)
    .addOptionalParam("factory", "mos contract address",DEPLOY_FACTORY , types.string)