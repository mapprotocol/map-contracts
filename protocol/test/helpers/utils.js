const { ethers } = require("hardhat");
const chai = require("chai");
const { solidity } = require("ethereum-waffle");

chai.use(solidity); // BigNumber equality, .emit / .withArgs
const { expect } = chai;

/// Await a tx/call that is expected to revert. `msg` (optional) must appear in the revert reason.
async function expectRevert(promise, msg) {
    let err;
    try {
        const r = await promise;
        if (r && r.wait) await r.wait();
    } catch (e) {
        err = e;
    }
    expect(err, "expected the call to revert").to.not.equal(undefined);
    if (msg) {
        const text = (err.message || "") + JSON.stringify(err.error || "") + (err.reason || "");
        expect(text).to.include(msg);
    }
}

async function deployProxy(implName, proxyName, initData = "0x") {
    const Impl = await ethers.getContractFactory(implName);
    const impl = await Impl.deploy();
    await impl.deployed();
    const Proxy = await ethers.getContractFactory(proxyName);
    const proxy = await Proxy.deploy(impl.address, initData);
    await proxy.deployed();
    return { impl, proxy, contract: Impl.attach(proxy.address) };
}

const bn = (v) => ethers.BigNumber.from(v);

module.exports = { expectRevert, deployProxy, bn };
