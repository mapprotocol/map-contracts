import { HardhatUserConfig } from "hardhat/config";
import * as dotenv from "dotenv";
// import "@nomiclabs/hardhat-etherscan";
import "@nomiclabs/hardhat-waffle";
import 'hardhat-deploy';
import "hardhat-gas-reporter";
import "solidity-coverage";

dotenv.config();

const config: HardhatUserConfig ={
  solidity: {
		compilers: [
			{ version: "0.8.7", settings: { optimizer: { enabled: true, runs: 200 } } },
      { version: "0.8.17", settings: { optimizer: { enabled: true, runs: 200 } } },
		],
	},
  namedAccounts: {
    deployer: 0,
  },
  networks: {
    makalu: {
      chainId: 212,
      url:"https://testnet-rpc.maplabs.io",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    map : {
      chainId: 22776,
      url:"https://rpc.maplabs.io",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },

    platon : {
      chainId: 2206132,
      url:"https://devnet2openapi.platon.network/rpc",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },

    Goerli: {
      url: "https://goerli.blockpi.network/v1/rpc/public",
      chainId : 5,
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },

    platon_main : {
      chainId: 210425,
      url:"https://openapi2.platon.network/rpc",
      accounts: process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    
  },
  gasReporter: {
    enabled: process.env.REPORT_GAS !== undefined,
    currency: "USD",
  },
  // etherscan: {
  //   apiKey: process.env.ETHERSCAN_API_KEY,
  // },
};

export default config;




