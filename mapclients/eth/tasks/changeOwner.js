module.exports = async (taskArgs, hre) => {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let proxy = await ethers.getContractAt("LightNode", taskArgs.node);

    console.log("pre admin:", await proxy.connect(deployer).getAdmin());

    await (await proxy.connect(deployer).setPendingAdmin(taskArgs.owner)).wait();

    console.log(`LightNode set pending admin : ${taskArgs.owner} `);
};
