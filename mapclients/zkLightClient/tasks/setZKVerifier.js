module.exports = async (taskArgs, hre) => {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let LightNodeProxy = await deployments.get("LightNodeProxy");

    let proxy = await ethers.getContractAt("LightNode", LightNodeProxy.address);

    await (await proxy.connect(deployer).setZKVerifier(taskArgs.verifier)).wait();

    console.log(`LightNode setVerifyTool is : ${taskArgs.verifier} `);
};
