import { HardhatUserConfig } from "hardhat/config";
import * as dotenv from "dotenv";
import "@nomiclabs/hardhat-etherscan";
import "@nomiclabs/hardhat-waffle";
import 'hardhat-deploy';
import "hardhat-gas-reporter";
import "solidity-coverage";

require("./tasks");

dotenv.config();

const config: HardhatUserConfig ={
  solidity: {
		compilers: [
			{ version: "0.8.7", settings: { optimizer: { enabled: true, runs: 200 } } },
		],
	},
  namedAccounts: {
    deployer: 0,
  },
  networks: {
    Makalu: {
      chainId: 212,
      url:"https://testnet-rpc.maplabs.io",
      accounts: process.env.TESTNET_PRIVATE_KEY !== undefined ? [process.env.TESTNET_PRIVATE_KEY] : [],
    },
    MakaluArbitrum: {
      chainId: 212,
      url:"https://testnet-rpc.maplabs.io",
      accounts: process.env.TESTNET_PRIVATE_KEY !== undefined ? [process.env.TESTNET_PRIVATE_KEY] : [],
    },
    map_op : {
      chainId: 22776,
      url:"https://rpc.maplabs.io",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    map_arb : {
      chainId: 22776,
      url:"https://rpc.maplabs.io",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },

    Arbitrum: {
      chainId: 42161,
      url: `https://1rpc.io/arb`,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },

    ArbitrumSepolia: {
      chainId: 421614,
      url: `https://arbitrum-sepolia.blockpi.network/v1/rpc/public`,
      accounts: process.env.TESTNET_PRIVATE_KEY !== undefined ? [process.env.TESTNET_PRIVATE_KEY] : [],
    },
    Tron: {
      url: `https://testnet-rpc.maplabs.io`,
      chainId: 212, //728126428,
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    TronTest: {
      url: `https://testnet-rpc.maplabs.io`,
      chainId: 212,
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
  },
  // gasReporter: {
  //   enabled: process.env.REPORT_GAS !== undefined,
  //   currency: "USD",
  // },
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY,
  },
};

export default config;




