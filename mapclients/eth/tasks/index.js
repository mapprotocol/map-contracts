
task("initializeData",
    "Write initialization data required by LightNode",
    require("./initializeData")
)
    .addParam("epoch", "The account's address")
