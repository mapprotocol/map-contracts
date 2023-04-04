const { LIGHTNODE_SALT,DEPLOY_FACTORY} = process.env;


task("lightProxy",
    "deploy LightNode proxy and init",
    require("./lightProxy")
)
    .addParam("height", "init height")
    .addParam("rpc", "main or test")


task("lightFactoryDeploy",
    "deploy LightNode proxy and init",
    require("./lightFactoryDeploy")
)
    .addParam("height", "init height")
    .addParam("mpt", "MPT verify address")
    .addOptionalParam("salt", "deploy contract salt",LIGHTNODE_SALT , types.string)
    .addOptionalParam("factory", "mos contract address",DEPLOY_FACTORY , types.string)