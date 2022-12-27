
task("LightClientRegister",
    "Deploy LightClientRegister",
    require("./clientRegister")
)
    .addParam("chain", "chain id for light client")
    .addParam("contract", "contract for light client")

task("MaintainerWhileListSet",
    "Deploy MaintainerManagerSet",
    require("./MaintainerWhileListSet")
)
    .addParam("add", "add:true remove:false")
    .addParam("address", "Maintainer address")

task("clientGetRange",
    "Get light client verifiable range",
    require("./clientGetRange")
)
    .addOptionalParam("manager", "light client manager address", "")
    .addParam("chain", "light client chain id")