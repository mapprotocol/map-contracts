
const chainlist = [1,
    56, 97,  // bsc
    137, 80001, // matic
    212, 22776,  // map
    "5566818579631833088", "5566818579631833089" // near
];


module.exports = async (taskArgs,hre) => {
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    console.log("deployer address:",deployer.address);

    let address = taskArgs.relay;
    if (address == "relay") {
        let proxy = await hre.deployments.get("MAPOmnichainServiceProxyV2")

        address = proxy.address;
    }
    console.log("mos address:\t", address);

    let mos = await ethers.getContractAt('MAPOmnichainServiceRelayV2', address);

    let tokenmanager = await mos.tokenRegister();
    let wtoken = await mos.wToken();
    let selfChainId = await mos.selfChainId();
    let lightClientManager = await mos.lightClientManager();

    let vaultFee = await mos.distributeRate(0);
    let mistFee = await mos.distributeRate(1);

    console.log("selfChainId:\t", selfChainId.toString());
    console.log("light client manager:", lightClientManager);
    console.log("Token manager:\t", tokenmanager);
    console.log("wToken address:\t", wtoken);

    console.log(`distribute rate: vault(${vaultFee[1]}), receiver(${mistFee[0]}), rate(${mistFee[1]})`);

    let manager = await ethers.getContractAt('TokenRegisterV2', tokenmanager);

    console.log("\nRegister chains:");
    let chains = [selfChainId];
    for (let i = 0; i < chainlist.length; i++) {
        let contract = await mos.mosContracts(chainlist[i]);

        if (contract != "0x") {
            let chaintype = await mos.chainTypes(chainlist[i]);
            console.log(`type(${chaintype}) ${chainlist[i]}\t => ${contract} `);
            chains.push(chainlist[i]);
        }
    }

    address = taskArgs.token;
    if (address == "wtoken") {
        address = wtoken;
    }
    console.log("\ntoken address:", address);
    let token = await manager.tokenList(address);
    console.log(`token mintalbe:\t ${token.mintable}`);
    console.log(`token decimals:\t ${token.decimals}`);
    console.log(`vault address: ${token.vaultToken}`);

    let vault = await ethers.getContractAt('VaultTokenV2', token.vaultToken);
    let totalVault = await vault.totalVault();
    console.log(`total vault:\t ${totalVault}`);

    console.log(`chains:`);
    for (let i = 0; i < chains.length; i++) {
        let info = await manager.getToChainTokenInfo(address, chains[i]);
        console.log(`${chains[i]}\t => ${info[0]} (${info[1]}), `);

        let balance = await vault.vaultBalance(chains[i]);
        console.log(`\t vault(${balance}), fee min(${info[2][0]}), max(${info[2][1]}), rate(${info[2][2]})`);
    }

}