module.exports = async (taskArgs, hre) => {
    const { deploy } = deployments;
    const accounts = await ethers.getSigners();
    const deployer = accounts[0];

    console.log("Deploying contracts with the account:", await deployer.getAddress());

    console.log("Account balance:", (await deployer.getBalance()).toString());

    let factory = await ethers.getContractAt("IDeployFactory", taskArgs.factory);

    console.log("deploy factory address:", factory.address);

    console.log("mpt salt:", taskArgs.salt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.salt));

    let mpt = await ethers.getContractFactory("MPTVerify");

    let deployData = mpt.bytecode;

    await (await factory.connect(deployer).deploy(hash, deployData, 0)).wait();

    let mptAddress = await factory.connect(deployer).getAddress(hash);

    console.log("deployed mpt address:", mptAddress);
};
