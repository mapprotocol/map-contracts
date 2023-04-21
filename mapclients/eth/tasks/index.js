const { LIGHTNODE_SALT,VERIFYTOOL_SALT,DEPLOY_FACTORY} = process.env;

task("initializeData",
    "Write initialization data required by LightNode",
    require("./initializeData")
)
    .addParam("epoch", "The epoch number")


task("lightNodeDeploy",
    "Write initialization data required by LightNode",
    require("./lightNodeDeploy")
)
    .addParam("verify", "The epoch number")
    .addOptionalParam("salt", "deploy contract salt",LIGHTNODE_SALT , types.string)
    .addOptionalParam("factory", "mos contract address",DEPLOY_FACTORY , types.string)


task("verifyToolDeploy",
    "Write initialization data required by LightNode",
    require("./verifyToolDeploy")
)
    .addOptionalParam("toolsalt", "deploy contract salt",VERIFYTOOL_SALT , types.string)
    .addOptionalParam("factory", "mos contract address",DEPLOY_FACTORY , types.string)
