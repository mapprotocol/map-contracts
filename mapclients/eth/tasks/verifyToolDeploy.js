
module.exports = async (taskArgs, hre) => {
    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:", deployer.address);

    let VerifyTool = await ethers.getContractFactory('VerifyTool');

    let deployData = VerifyTool.bytecode

    console.log("mos salt:", taskArgs.toolsalt);

    let hash = await ethers.utils.keccak256(await ethers.utils.toUtf8Bytes(taskArgs.toolsalt));

    let factory = await ethers.getContractAt("IDeployFactory",taskArgs.factory)

    console.log("deploy factory address:",factory.address)

    await (await factory.connect(deployer).deploy(hash,deployData,0)).wait();

    let verifyToolAddress = await factory.connect(deployer).getAddress(hash)

    console.log(`VerifyTool  contract address is ${verifyToolAddress}`)

}
