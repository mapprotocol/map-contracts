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
//    currency: 'CNY',
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
        //url: `https://bsctest.pls2e.cc`,
        url: `https://data-seed-prebsc-1-s1.binance.org:8545`
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
    },
    MapDev: {
      url: `http://3.0.19.66:7445/`,
      chainId : 213,
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
      url: `https://public-en-cypress.klaytn.net`,
      chainId : 8217,
      accounts: accounts
    },
    KlayTest: {
      url: `https://public-en-baobab.klaytn.net`,
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
    },

    Conflux: {
      url: `https://evm.confluxrpc.com`,
      chainId : 1030,
      accounts: accounts
    },
    ConfluxTest: {
      url: `https://evmtestnet.confluxrpc.com`,
      chainId : 71,
      accounts: accounts
    },

    Tron: {
      url: `https://api.trongrid.io`,
      chainId : 728126428,
      accounts: accounts
    },

    TronTest: {
      url: `https://nile.trongrid.io`,
      chainId : 3448148188,
      accounts: accounts
    },

    Avax: {
      url: `https://rpc.ankr.com/avalanche`,
      chainId : 43114,
      accounts: accounts
    },
    Filecoin: {
      url: `https://rpc.ankr.com/filecoin`,
      chainId : 314,
      accounts: accounts
    },

    Arbitrum: {
      url: `https://1rpc.io/arb`,
      chainId : 42161,
      accounts: accounts
    },
    ArbitrumGoerli: {
      url: `https://arbitrum-goerli.infura.io/v3/` + INFURA_KEY,
      chainId : 421613,
      accounts: accounts
    },

    zkSync: {
      url: `https://mainnet.era.zksync.io`,
      chainId : 324,
      zksync: true,
      ethNetwork: 'Eth',
      accounts: accounts
    },
    Optimism: {
      url: `https://1rpc.io/op`,
      chainId : 10,
      accounts: accounts
    },
    Base: {
      url: `https://mainnet.base.org`,
      chainId : 8453,
      accounts: accounts
    },
    zkEvm: {
      url: `https://zkevm-rpc.com`,
      chainId : 1101,
      accounts: accounts
    },
    Linea: {
      url: `https://rpc.linea.build`,
      chainId : 59144,
      accounts: accounts
    },
    Scroll: {
      url: `https://rpc.scroll.io`,
      chainId : 534352,
      accounts: accounts
    },
    Boba: {
      url: `https://mainnet.boba.network`,
      chainId : 288,
      accounts: accounts
    },
    Metis: {
      url: `https://andromeda.metis.io/?owner=1088`,
      chainId : 1088,
      accounts: accounts
    },
    Mantle: {
      url: `https://rpc.mantle.xyz`,
      chainId : 5000,
      accounts: accounts
    },

  },
  solidity: {
    compilers: [
      {
        version: '0.8.7',
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
