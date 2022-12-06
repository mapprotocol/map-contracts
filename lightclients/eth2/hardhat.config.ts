import {HardhatUserConfig} from "hardhat/config";
import * as dotenv from "dotenv";
import "@nomiclabs/hardhat-etherscan";
import "@nomiclabs/hardhat-waffle";
import 'hardhat-deploy';
import "hardhat-gas-reporter";
import "solidity-coverage";

dotenv.config();

const config: HardhatUserConfig = {
    defaultNetwork: process.env.DEFAULT_NETWORK != undefined ? process.env.DEFAULT_NETWORK : 'hardhat',
    solidity: {
        compilers: [
            {version: "0.8.7", settings: {optimizer: {enabled: true, runs: 200}}},
        ],
    },
    namedAccounts: {
        deployer: 0,
    },
    networks: {
        local: {
            chainId: 214,
            url: "http://localhost:7445",
            accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
        },
        hardhat: {
            gas: 20000000,
            chainId: 43112,
        },
        dev: {
            chainId: 212,
            url: "http://3.0.19.66:7445",
            accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
        },
        makalu: {
            chainId: 212,
            url: "https://testnet-rpc.maplabs.io",
            accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
        },
        map: {
            chainId: 22776,
            url: "https://rpc.maplabs.io",
            accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
        },

    },
    gasReporter: {
        enabled: process.env.REPORT_GAS !== undefined,
        currency: "USD",
    },
    etherscan: {
        apiKey: process.env.ETHERSCAN_API_KEY,
    },
};

export default config;




