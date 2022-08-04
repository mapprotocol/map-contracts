require("@nomicfoundation/hardhat-toolbox");
require("dotenv").config();



module.exports = {
  solidity: {
		compilers: [
			{ version: "0.8.4", settings: { optimizer: { enabled: true, runs: 200 } } },
		],
	},
  networks: {
    ropsten: {
      url: process.env.ROPSTEN_URL || "",
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    map_test: {
      chainId: 212,
      url: process.env.ROPSTEN_URL || "http://18.142.54.137:7445",
      accounts: { mnemonic: process.env.MNEMONIC},
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


