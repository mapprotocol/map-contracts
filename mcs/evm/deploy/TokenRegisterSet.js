
let tokenRegisterAddress = "0x9B12Acf2C97Fc939f29Ee0BD0083Ec29C4F00BE3"

let ethUsdt = "0xdBf63a81d44DA9645498E371A856F9754F4f2c2B";
let nearUsdt = "0x6d63735f746f6b656e5f312e6d63732e6d61703030312e746573746e6574";
let mapUsdt = "0xa2FD5Ad95c1F2fC374dF775ad0889eab6d587015";

module.exports = async function ({ethers, deployments}) {
    const {deploy} = deployments
    const {deployer} = await ethers.getNamedSigners()

    console.log("deployer address:",deployer.address);

    let tokenRegister = await ethers.getContractAt('TokenRegister',tokenRegisterAddress);


    //tokenRegister set regToken
    await (await tokenRegister.connect(deployer).regToken(34434,ethUsdt,mapUsdt)).wait();

    await (await tokenRegister.connect(deployer).regToken(1313161555,nearUsdt,mapUsdt)).wait();

    await (await tokenRegister.connect(deployer).regToken(212,mapUsdt,ethUsdt)).wait();
    console.log("tokenRegister set success");

}

module.exports.tags = ['TokenRegisterSet']