const { LIGHTNODE_SALT,DEPLOY_FACTORY} = process.env;


task("lightFactoryDeploy",
    "deploy LightNode proxy and init",
    require("./lightFactoryDeploy")
)
    .addParam("height", "init height")
    .addParam("tool", " verify tool address")
    .addOptionalParam("salt", "deploy contract salt",LIGHTNODE_SALT , types.string)
    .addOptionalParam("factory", "mos contract address",DEPLOY_FACTORY , types.string)