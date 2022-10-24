
task("LightClientRegister",
    "Deploy LightClientRegister",
    require("./LightClientRegister")
)
    .addParam("chain", "chain id for light client")
    .addParam("contract", "contract for light client")

task("MaintainerWhileListSet",
    "Deploy MaintainerManagerSet",
    require("./MaintainerWhileListSet")
)
    .addParam("add", "add:true remove:false")
    .addParam("address", "Maintainer address")