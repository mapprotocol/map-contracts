require("@nomiclabs/hardhat-waffle");
require('hardhat-gas-reporter')
require('hardhat-spdx-license-identifier')
require('hardhat-deploy')
require('hardhat-abi-exporter')
require('@nomiclabs/hardhat-ethers')
require('dotenv/config')
require('@nomiclabs/hardhat-etherscan')
require('solidity-coverage')
require('./tasks')

const { PRIVATE_KEY, INFURA_KEY} = process.env;

let accounts = [];

accounts.push(PRIVATE_KEY);

module.exports = {
  defaultNetwork: 'hardhat',
  gasReporter: {
    enabled: false,
  },
  abiExporter: {
    path: './abi',
    clear: false,
    flat: true
  },
  networks: {
    hardhat: {
      forking: {
        enabled: false,
        url: `https://rpc.maplabs.io/`,
        //url: `https://data-seed-prebsc-1-s1.binance.org:8545`
        //url: `https://bsc-dataseed.eme-node.com`,
        //url: `https://bsc-dataseed2.defibit.io/`,
      },
      live: true,
      saveDeployments: false,
      tags: ['local'],
      timeout: 2000000
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
    }
  },
  solidity: {
    compilers: [
      {
        version: '0.8.12',
        settings: {
          optimizer: {
            enabled: true,
            runs: 200
          }
        }
      },
      {
        version: '0.4.22',
        settings: {
          optimizer: {
            enabled: true,
            runs: 200
          }
        }
      }
    ]
  },
  spdxLicenseIdentifier: {
    overwrite: true,
    runOnCompile: false
  },
  mocha: {
    timeout: 2000000
  },
  etherscan: {
    apiKey: process.env.BSC_SCAN_KEY
  }
}
