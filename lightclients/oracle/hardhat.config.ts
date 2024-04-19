import { HardhatUserConfig } from "hardhat/config";
import * as dotenv from "dotenv";
import "@nomiclabs/hardhat-etherscan";
import "@nomiclabs/hardhat-waffle";
import 'hardhat-deploy';
import "hardhat-gas-reporter";
import "solidity-coverage";

import "@matterlabs/hardhat-zksync-deploy";
import "@matterlabs/hardhat-zksync-solc";
import "@matterlabs/hardhat-zksync-verify";

require("./tasks");

dotenv.config();

const config: HardhatUserConfig ={
  zksolc: {
    version: "1.4.0",
    compilerSource: "binary",
    settings: {},
  },
  solidity: {
		compilers: [
			{ version: "0.8.20", settings: { "evmVersion": "london" } },
		],
	},
  namedAccounts: {
    deployer: 0,
  },
  networks: {
    Mapo : {
      chainId: 22776,
      url:"https://rpc.maplabs.io",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    Eth: {
      chainId: 1,
      url: `https://1rpc.io/eth`,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    Arbitrum: {
      chainId: 42161,
      url: `https://1rpc.io/arb`,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    Blast: {
      url: `https://rpc.blast.io`,
      chainId : 81457,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    Merlin: {
      url: `https://rpc.merlinchain.io`,
      chainId: 4200,
      gasPrice: 50000000,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    Tron: {
      url: `https://api.trongrid.io/jsonrpc`,
      chainId: 728126428,
      accounts:
          process.env.TRON_PRIVATE_KEY !== undefined ? [process.env.TRON_PRIVATE_KEY] : [],
    },
    Ainn: {
      url: `https://mainnet-rpc.anvm.io`,
      chainId : 2649,
      gasPrice: 50000000,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    zkSync: {
      url: `https://mainnet.era.zksync.io`,
      chainId: 324,
      zksync: true,
      ethNetwork: "Eth",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },

    Makalu: {
      chainId: 212,
      url:"https://testnet-rpc.maplabs.io",
      accounts: process.env.TESTNET_PRIVATE_KEY !== undefined ? [process.env.TESTNET_PRIVATE_KEY] : [],
    },

    ArbitrumSepolia: {
      chainId: 421614,
      url: `https://arbitrum-sepolia.blockpi.network/v1/rpc/public`,
      accounts: process.env.TESTNET_PRIVATE_KEY !== undefined ? [process.env.TESTNET_PRIVATE_KEY] : [],
    },

    TronTest: {
      url: `https://nile.trongrid.io/jsonrpc`,
      chainId: 3448148188,
      accounts:
        process.env.TRON_PRIVATE_KEY !== undefined ? [process.env.TRON_PRIVATE_KEY] : [],
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




