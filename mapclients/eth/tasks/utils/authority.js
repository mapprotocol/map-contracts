const { ethers } = require("hardhat");

let Authority_addr = "0xAaaAa8a316ab372Af9BC4cDD2ae040b03f9D4d88";
let Authority_abi = [
    "function isAuthorized(address user, address target, bytes4 funSig) external view returns (bool)",
    "function getRole(address target, bytes4 funSig) external view returns (bytes32)",
    "function execute(address target, uint256 value, bytes calldata payload) external payable",
    "function addControl(address target, bytes4 funSig, bytes32 role) external",
];

async function addControl(contract, functionName, role, wallet, authority_addr = Authority_addr) {
    let authority = await ethers.getContractAt(Authority_abi, authority_addr);
    let target = contract.address;
    let func = contract.interface.getSighash(functionName);

    console.log("target: ", target);
    console.log("function: ", functionName);
    console.log("role: ", role);

    role = getRoleBytes(role);

    let data = authority.interface.encodeFunctionData("addControl", [target, func, role]);

    console.log("execute data: ", data);
    let result = false;
    if (wallet) {
        let tx = {
            to: authority.address,
            data: data,
        };
        let res = await (await wallet.sendTransaction(tx)).wait();
        result = res.status === 1;
    }

    return result, data;
}

async function execute(contract, functionName, args, wallet, value = 0, authority_addr = Authority_addr) {
    let authority = await ethers.getContractAt(Authority_abi, authority_addr);
    let target = contract.address;
    let playload = contract.interface.encodeFunctionData(functionName, args);
    let data = authority.interface.encodeFunctionData("execute", [target, value, playload]);
    console.log("execute data: ", data);
    let result = false;
    if (wallet) {
        let tx = {
            to: authority.address,
            value: value,
            data: data,
        };
        let res = await (await wallet.sendTransaction(tx)).wait();
        result = res.status === 1;
    }
    return result, data;
}

async function isAuthorized(user, contract, functionName, authority_addr = Authority_addr) {
    let authority = await ethers.getContractAt(Authority_abi, authority_addr);
    let target = contract.address;
    let func = contract.interface.getSighash(functionName);
    return await authority.isAuthorized(user, target, func);
}

async function getRole(contract, functionName, authority_addr = Authority_addr) {
    //let authority = new ethers.Contract(authority_addr,Authority_abi);
    let authority = await ethers.getContractAt(Authority_abi, authority_addr);
    let target = contract.address;
    let func = contract.interface.getSighash(functionName);
    return await authority.getRole(target, func);
}

function getRoleBytes(role) {
    if (role.startsWith("0x")) {
        return role;
    }
    if (role === "DEFAULT_ADMIN_ROLE") {
        return ethers.constants.HashZero;
    } else {
        return ethers.utils.keccak256(ethers.utils.toUtf8Bytes(taskArgs.role));
    }
}

module.exports = {
    addControl,
    execute,
    isAuthorized,
    getRole,
};
