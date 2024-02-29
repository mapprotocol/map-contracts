module.exports = async (taskArgs, hre) => {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let factory = await ethers.getContractAt("IDeployFactory", taskArgs.factory);

    console.log("deploy factory address:", factory.address);

    await (await factory.connect(deployer).deploy(hash, deployData, 0)).wait();

    let lightProxyAddress = await factory.connect(deployer).getAddress(hash);

    let proxy = await ethers.getContractAt("LightNode", lightProxyAddress);

    await (await proxy.connect(deployer).setLedgerInfo(taskArgs.ledger)).wait();

    console.log(`LightNode setVerifyTool is : ${taskArgs.ledger} `);
};
