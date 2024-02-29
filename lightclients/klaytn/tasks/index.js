const { LIGHTNODE_SALT, DEPLOY_FACTORY, MPT_VERIFIER } = process.env;

task("lightFactoryDeploy", "deploy LightNode proxy and init", require("./lightFactoryDeploy"))
    .addParam("height", "init height")
    .addParam("tool", " verify tool address")
    .addOptionalParam("mpt", "mpt verifier contract address", MPT_VERIFIER, types.string)
    .addOptionalParam("salt", "deploy contract salt", LIGHTNODE_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);

task("setVerifyTool", "LightNode set the verifyTool contract address", require("./setVerifyTool")).addParam(
    "tool",
    "verifyTool contract address"
);

task("factorySetVerifyTool", "LightNode set the verifyTool contract address", require("./factorySetVerifyTool"))
    .addParam("tool", "verifyTool contract address")
    .addOptionalParam("salt", "deploy contract salt", LIGHTNODE_SALT, types.string)
    .addOptionalParam("factory", "mos contract address", DEPLOY_FACTORY, types.string);
