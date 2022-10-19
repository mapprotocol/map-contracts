require("@nomicfoundation/hardhat-toolbox");
require('hardhat-deploy');
require("dotenv").config();



module.exports = {
  solidity: {
    compilers: [
      { version: "0.8.4", settings: { optimizer: { enabled: true, runs: 200 } } },
    ],
  },
  namedAccounts: {
    deployer: 0,
  },
  networks: {
    makalu: {
      chainId: 212,
      url: 'https://testnet-rpc.maplabs.io',
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
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


