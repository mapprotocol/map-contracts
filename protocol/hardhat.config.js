require('hardhat-gas-reporter');
require('hardhat-spdx-license-identifier');
require('hardhat-deploy');
require('hardhat-abi-exporter');
require('@nomiclabs/hardhat-ethers');
require('dotenv/config');
require('@nomiclabs/hardhat-etherscan');
require('hardhat-contract-sizer');
require("hardhat-log-remover");
require("./tasks")


const { PRIVATE_KEY, INFURA_KEY} = process.env;


let accounts = [];

accounts.push(PRIVATE_KEY);


module.exports = {
  defaultNetwork: 'hardhat',
  abiExporter: {
    path: './abi',
    clear: false,
    flat: true,
  },
  networks: {
    hardhat: {
      forking: {
        enabled: false,
        //url: `https://bsctest.pls2e.cc`,
        url: `https://data-seed-prebsc-1-s1.binance.org:8545`,
        //url: `https://bsc-dataseed.eme-node.com`,
        //url: `https://bsc-dataseed2.defibit.io/`,
      },
      live: true,
      saveDeployments: false,
      tags: ['local'],
      timeout: 2000000,
    },
    Map: {
      url: `https://rpc.maplabs.io/`,
      chainId : 22776,
      accounts: accounts
    },
    Makalu: {
      url: `https://testnet-rpc.maplabs.io/`,
      chainId : 212,
      accounts: accounts
    },
    Matic: {
      url: `https://rpc-mainnet.maticvigil.com`,
      chainId : 137,
      accounts: accounts
    },
    MaticTest: {
      url: `https://rpc-mumbai.maticvigil.com/`,
      chainId : 80001,
      accounts: accounts
    },
    Heco: {
      url: `https://http-mainnet-node.huobichain.com`,
      chainId : 128,
      accounts: accounts
    },
    HecoTest: {
      url: `https://http-testnet.hecochain.com`,
      chainId : 256,
      accounts: accounts
    },
    Bsc: {
      url: `https://bsc-dataseed1.binance.org/`,
      chainId : 56,
      accounts: accounts
    },
    BscTest: {
      url: `https://data-seed-prebsc-2-s1.binance.org:8545/`,
      chainId : 97,
      accounts: accounts,
      gasPrice: 11 * 1000000000
    },
    Klay: {
      url: `https://public-node-api.klaytnapi.com/v1/cypress`,
      chainId : 8217,
      accounts: accounts
    },
    KlayTest: {
      url: `https://api.baobab.klaytn.net:8651/`,
      chainId : 1001,
      accounts: accounts
    },
    Eth: {
      url: `https://mainnet.infura.io/v3/` + INFURA_KEY,
      chainId : 1,
      accounts: accounts
    },
    Goerli: {
      url: `https://goerli.infura.io/v3/` + INFURA_KEY,
      chainId : 5,
      accounts: accounts
    }
  },
  solidity: {
    compilers: [
      {
        version: '0.8.7',
        settings: {
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
      {
        version: '0.4.22',
        settings: {
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
    ],
  },
  spdxLicenseIdentifier: {
    overwrite: true,
    runOnCompile: true,
  },
  mocha: {
    timeout: 2000000,
  },
  etherscan: {
    apiKey: process.env.BSC_SCAN_KEY,
  },
}
