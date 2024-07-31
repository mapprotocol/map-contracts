module.exports = async (taskArgs, hre) => {
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];
    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let proxy = await ethers.getContract("LightClientManagerProxy");

    console.log("Light Clinet Manager Proxy", proxy.address);

    let manager = await ethers.getContractAt("LightClientManager", proxy.address);

    let admin = await manager.getAdmin();
    console.log(`change admin: ${admin} -> ${taskArgs.owner} `);

    await manager.changeAdmin(taskArgs.owner, { gasLimit: 100000 });

    console.log(`Change owner ${taskArgs.owner} successfully`);
};
