task("lightProxy",
    "deploy LightNode proxy and init",
    require("./lightProxy")
)
    .addParam("height", "init height")
    .addParam("rpc", "main or test")
