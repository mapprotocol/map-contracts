task("clientRegister", "Deploy LightClientRegister", require("./clientRegister"))
    .addParam("chain", "chain id for light client")
    .addParam("contract", "contract for light client");

task("deployMpt", "Deploy MPT verifier with factory", require("./deployMpt"))
    .addOptionalParam("factory", "mos contract address", "0x6258e4d2950757A749a4d4683A7342261ce12471", types.string)
    .addParam("salt", "mpt salt");

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

task("getDeployAddress2", "Get contract address deployed by DeployFactory ", require("./getDeployAddress2"))
    .addOptionalParam(
        "factory",
        "DeployFactory, default is 0x6258e4d2950757A749a4d4683A7342261ce12471",
        "0x6258e4d2950757A749a4d4683A7342261ce12471",
        types.string
    )
    .addOptionalParam("count", "salt count", 100, types.int)
    .addParam("salt", "contract salt")
    .addParam("match", "contract salt");
