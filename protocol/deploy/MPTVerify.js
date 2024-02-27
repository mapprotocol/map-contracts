module.exports = async function ({ ethers, deployments }) {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    await deploy("MPTVerify", {
        from: deployer.address,
        args: [],
        log: true,
        contract: "MPTVerify",
        deterministicDeployment: false,
    });

    let verifier = await ethers.getContract("MPTVerify");

    console.log("MPTVerify address:", verifier.address);
};

module.exports.tags = ["MPTVerify"];
