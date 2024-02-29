const { LIGHTNODE_SALT, VERIFYTOOL_SALT, DEPLOY_FACTORY } = process.env;

task("initializeData", "Write initialization data required by LightNode", require("./initializeData")).addParam(
    "epoch",
    "The epoch number"
);

task("lightNodeDeploy", "Write initialization data required by LightNode", require("./lightNodeDeploy"))
    .addParam("verify", "verify tool address")
    .addOptionalParam("salt", "deploy contract salt", LIGHTNODE_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);

task("verifyToolDeploy", "Write initialization data required by LightNode", require("./verifyToolDeploy"))
    .addOptionalParam("salt", "deploy contract salt", VERIFYTOOL_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);

task("setVerifyTool", "LightNode set the verifyTool contract address", require("./setVerifyTool")).addParam(
    "tool",
    "verifyTool contract address"
);

task("factorySetVerifyTool", "LightNode set the verifyTool contract address", require("./factorySetVerifyTool"))
    .addParam("tool", "verifyTool contract address")
    .addOptionalParam("salt", "deploy contract salt", LIGHTNODE_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);

task("tronDeploy", "Write initialization data required by LightNode", require("./tronDeploy"));
