const sleep = (delay) => new Promise((resolve) => setTimeout(resolve, delay));

exports.verify = async function (addr, args, code, chainId, wait) {
    if (needVerify(chainId)) {
        if (wait) {
            await sleep(20000);
        }
        console.log(`verify ${code} ...`);
        await run("verify:verify", {
            address: addr,
            constructorArguments: args,
            contract: code,
        });
    }
};

function needVerify(chainId) {
    let needs = [
        1, // eth
        56, // bsc
        137, // matic
        199, // bttc
        81457, // blast
        8453, // base
        324, // zksync
        10, // op
        42161, // arb
        59144, // linea
        534352, // scroll
        5000, // mantle
    ];
    if (needs.includes(chainId)) {
        return true;
    } else {
        return false;
    }
}
