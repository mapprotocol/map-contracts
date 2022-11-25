
module.exports = async (taskArgs) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceRelayV2', taskArgs.mos);

    let address = taskArgs.address;
    if (taskArgs.address === "") {
        address = deployer.address;
    }

    let token = taskArgs.token;
    if (taskArgs.token === "0x0000000000000000000000000000000000000000"){
        token = await mos.wToken();
    }
    let managerAddress = await mos.tokenRegister();
    let manager = await ethers.getContractAt('TokenRegisterV2', managerAddress);

    let vaultAddress = await manager.getVaultToken(token);

    console.log(`token address: ${token}, vault token address: ${vaultAddress}`);

    let vaultToken = await ethers.getContractAt("IERC20", vaultAddress);


    await (await mos.connect(deployer).withdraw(
        vaultAddress,
        taskArgs.value
    )).wait();

    console.log(`withdraw token ${token} from vault ${vaultAddress} ${taskArgs.value} to  ${address} successful`);
}