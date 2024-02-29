task("clientRegister", "Deploy LightClientRegister", require("./clientRegister"))
    .addParam("chain", "chain id for light client")
    .addParam("contract", "contract for light client");

task("clientGetRange", "Get light client verifiable range", require("./clientGetRange"))
    .addOptionalParam("manager", "light client manager address", "")
    .addParam("chain", "light client chain id");

task("getDeployAddress", "Get contract address deployed by DeployFactory ", require("./getDeployAddress"))
    .addOptionalParam(
        "factory",
        "DeployFactory, default is 0x6258e4d2950757A749a4d4683A7342261ce12471",
        "0x6258e4d2950757A749a4d4683A7342261ce12471",
        types.string
    )
    .addParam("salt", "contract salt");

task("MaintainerWhileListSet", "Deploy MaintainerManagerSet", require("./MaintainerWhileListSet"))
    .addParam("add", "add:true remove:false")
    .addParam("address", "Maintainer address");
