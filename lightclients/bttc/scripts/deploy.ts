import { ethers, run } from "hardhat";

//0x46AA055A9036Ae06A074f660FF60a5bCEd66f0f7
//relay 0x4f40F6b73dc4ABF49165387c00DE70d213558B93

async function main() {
    await deposit();
}

async function relayTo() {
    let [wallet] = await ethers.getSigners();
    const Relay = await ethers.getContractFactory("Relay");
    let relay = Relay.attach("0x4f40F6b73dc4ABF49165387c00DE70d213558B93");

    let depositData =
        "0x0000000000000000000000000000000000000000000000000de0b6b3a764000000000000000000000000000000000000000000000000000000000000000000d40000000000000000000000000000000000000000000000000000000000000061a367972f8fc399444443c569ac99a4e5f743a0d11e97fb180ec8ec10d5cc91700000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000de0b6b3a764000000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000001446aa055a9036ae06a074f660ff60a5bced66f0f700000000000000000000000000000000000000000000000000000000000000000000000000000000000000144f40f6b73dc4abf49165387c00de70d213558b9300000000000000000000000000000000000000000000000000000000000000000000000000000000000000146e3d5cb4a4f06c5d3c8017065029ee64c31d25a10000000000000000000000000000000000000000000000000000000000000000000000000000000000000020a367972f8fc399444443c569ac99a4e5f743a0d11e97fb180ec8ec10d5cc9170";

    await (await relay.relay(depositData)).wait();
}

async function deposit() {
    let [wallet] = await ethers.getSigners();
    const ChildERC20 = await ethers.getContractFactory("ChildERC20");
    let childERC20 = ChildERC20.attach("0x46AA055A9036Ae06A074f660FF60a5bCEd66f0f7");

    let amount = ethers.utils.parseEther("100");

    let e = ethers.utils.defaultAbiCoder.encode(
        ["uint256", "uint256", "bytes32", "bytes", "bytes", "bytes", "uint256", "bytes"],
        [
            "212",
            "97",
            "0xa367972f8fc399444443c569ac99a4e5f743a0d11e97fb180ec8ec10d5cc9170",
            "0x46AA055A9036Ae06A074f660FF60a5bCEd66f0f7",
            "0x4f40F6b73dc4ABF49165387c00DE70d213558B93",
            wallet.address,
            amount,
            "0xa367972f8fc399444443c569ac99a4e5f743a0d11e97fb180ec8ec10d5cc9170",
        ]
    );

    let depositData = ethers.utils.solidityPack(["uint256", "bytes"], [amount, e]);

    console.log(depositData);

    //  await (await childERC20.deposit(wallet.address,depositData)).wait()
}

async function deployChildToken() {
    let [wallet] = await ethers.getSigners();
    const ChildERC20 = await ethers.getContractFactory("ChildERC20");
    let name = "Mock Token";
    let symbol = "MT";
    let childManager = wallet.address;
    const childERC20 = await ChildERC20.deploy(name, symbol, childManager);
    await childERC20.connect(wallet).deployed();
    console.log("child", childERC20.address);
}

async function deployRelay() {
    let [wallet] = await ethers.getSigners();
    const Relay = await ethers.getContractFactory("Relay");
    let childToken = "0x46AA055A9036Ae06A074f660FF60a5bCEd66f0f7";
    const relay = await Relay.deploy(childToken);
    await relay.connect(wallet).deployed();
    console.log("relay", relay.address);
}

async function verify(addr: string, arg: Array<any>, code: string) {
    // await verify("0x3067c49494d25BF468d5eef7d9937a2fa0d5cC0E",[],"contracts/Mocks/MockUSDT18.sol:MockUSDT18")
    await run("verify:verify", {
        address: addr,
        constructorArguments: arg,
        contract: code,
    });
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
