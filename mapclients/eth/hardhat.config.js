require("@nomiclabs/hardhat-waffle");
require("hardhat-gas-reporter");
require("hardhat-spdx-license-identifier");
require("hardhat-deploy");
require("hardhat-deploy-ethers")
require("hardhat-abi-exporter");
require("@nomiclabs/hardhat-ethers");
require("dotenv/config");
require("@nomiclabs/hardhat-etherscan");
require("solidity-coverage");
require("@matterlabs/hardhat-zksync-deploy");
require("@matterlabs/hardhat-zksync-solc");
require("./tasks");

const { PRIVATE_KEY, INFURA_KEY } = process.env;

let accounts = [];

accounts.push(PRIVATE_KEY);

module.exports = {
  defaultNetwork: "hardhat",
  gasReporter: {
    enabled: false,
    //    currency: 'CNY',
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
      tags: ["local"],
      timeout: 2000000,
    },
    Map: {
      url: `https://rpc.maplabs.io/`,
      chainId: 22776,
      accounts: accounts,
    },
    Makalu: {
      url: `https://testnet-rpc.maplabs.io/`,
      chainId: 212,
      accounts: accounts,
    },
    MapDev: {
      url: `http://3.0.19.66:7445/`,
      chainId: 213,
      accounts: accounts,
    },
    Matic: {
      url: `https://polygon-mainnet.public.blastapi.io`,
      chainId: 137,
      accounts: accounts,
    },
    MaticTest: {
      url: `https://rpc-mumbai.maticvigil.com/`,
      chainId: 80001,
      accounts: accounts,
    },
    Bsc: {
      url: `https://bsc-dataseed1.binance.org/`,
      chainId: 56,
      accounts: accounts,
    },
    BscTest: {
      url: `https://data-seed-prebsc-2-s1.binance.org:8545/`,
      chainId: 97,
      accounts: accounts,
      gasPrice: 11 * 1000000000,
    },
    Klay: {
      url: `https://public-en-cypress.klaytn.net`,
      chainId: 8217,
      accounts: accounts,
    },
    KlayTest: {
      url: `https://public-en-baobab.klaytn.net`,
      chainId: 1001,
      accounts: accounts,
    },
    Eth: {
      url: `https://mainnet.infura.io/v3/` + INFURA_KEY,
      chainId: 1,
      accounts: accounts,
    },
    Sepolia: {
      url: `https://sepolia.drpc.org`,
      chainId: 11155111,
      accounts: accounts,
    },

    Conflux: {
      url: `https://evm.confluxrpc.com`,
      chainId: 1030,
      accounts: accounts,
    },
    ConfluxTest: {
      url: `https://evmtestnet.confluxrpc.com`,
      chainId: 71,
      accounts: accounts,
    },

    Tron: {
      url: `https://api.trongrid.io`,
      chainId: 728126428,
      accounts: accounts,
    },

    TronTest: {
      url: `https://nile.trongrid.io`,
      chainId: 3448148188,
      accounts: accounts,
    },

    zkEvm: {
      url: `https://zkevm-rpc.com`,
      chainId : 1101,
      accounts: accounts,
    },

    zkEvmTest: {
      url: `https://rpc.public.zkevm-test.net`,
      chainId : 1442,
      accounts: accounts,
    },

    Merlin: {
      url: `https://rpc.merlinchain.io/`,
      chainId: 4200,
      gasPrice: 50000000,
      accounts: accounts,
    },

    Bevm: {
      url: `https://rpc-canary-2.bevm.io/`,
      chainId : 1501,
      accounts: accounts,
    },

    Blast: {
      url: `https://rpc.blast.io`,
      chainId : 81457,
      accounts: accounts,
    },
    Ainn: {
      url: `https://mainnet-rpc.anvm.io`,
      chainId : 2649,
      gasPrice: 50000000,
      accounts: accounts,
    },
    B2: {
      url: `https://rpc.bsquared.network`,
      chainId : 223,
      gasPrice: 10000,
      accounts: accounts,
    },

    Avax: {
      url: `https://rpc.ankr.com/avalanche`,
      chainId: 43114,
      accounts: accounts,
    },
    Filecoin: {
      url: `https://rpc.ankr.com/filecoin`,
      chainId: 314,
      accounts: accounts,
    },

    Arbitrum: {
      url: `https://arb1.arbitrum.io/rpc`,
      chainId: 42161,
      accounts: accounts,
    },
    ArbitrumGoerli: {
      url: `https://arbitrum-goerli.infura.io/v3/` + INFURA_KEY,
      chainId: 421613,
      accounts: accounts,
    },

    zkSync: {
      url: `https://mainnet.era.zksync.io`,
      chainId: 324,
      zksync: true,
      ethNetwork: "Eth",
      accounts: accounts,
    },
    Optimism: {
      url: `https://1rpc.io/op`,
      chainId: 10,
      accounts: accounts,
    },
    Base: {
      url: `https://mainnet.base.org`,
      chainId: 8453,
      accounts: accounts,
    },
    zkEvm: {
      url: `https://zkevm-rpc.com`,
      chainId: 1101,
      accounts: accounts,
    },
    Linea: {
      url: `https://rpc.linea.build`,
      chainId: 59144,
      accounts: accounts,
    },
    Scroll: {
      url: `https://1rpc.io/scroll`,
      chainId: 534352,
      accounts: accounts,
    },
    Boba: {
      url: `https://mainnet.boba.network`,
      chainId: 288,
      accounts: accounts,
    },
    Metis: {
      url: `https://andromeda.metis.io/?owner=1088`,
      chainId: 1088,
      accounts: accounts,
    },
    Mantle: {
      url: `https://rpc.mantle.xyz`,
      chainId: 5000,
      accounts: accounts,
    },
  },
  zksolc: {
    version: "1.4.0",
    compilerSource: "binary",
    settings: {},
  },
  solidity: {
    compilers: [
      {
        version: "0.8.20",
        settings: {
          optimizer: {
            enabled: true,
            runs: 200
          },
          "evmVersion": "london"
        },
      },

    ],
  },
  spdxLicenseIdentifier: {
    overwrite: true,
    runOnCompile: false,
  },
  mocha: {
    timeout: 2000000,
  },
  etherscan: {
    apiKey: {
      Bttc: process.env.API_KEY_BTTC,
      Eth:  process.env.API_KEY_ETH,
      Bsc:  process.env.API_KEY_BSC,
      polygon: process.env.API_KEY_MATIC,
      Blast: process.env.API_KEY_BLAST,
      Base: process.env.API_KEY_BASE,
      zkSync: process.env.API_KEY_ZKSYNC,
      Optimism: process.env.API_KEY_OP,
      Arbitrum: process.env.API_KEY_ARBITRUM,
      Linea: process.env.API_KEY_LINEA,
      Scroll: process.env.API_KEY_SCROLL,
      Mantle: process.env.API_KEY_MANTLE
    },
    customChains: [
      {
        network: "Bttc",
        chainId: 199,
        urls: {
          apiURL: "https://api.bttcscan.com/api",
          browserURL: "https://bttcscan.com/",
        },
      },
      {
        network: "Eth",
        chainId: 1,
        urls: {
          apiURL: "https://api.etherscan.io/api",
          browserURL: "https://etherscan.com/",
        },
      },
      {
        network: "Bsc",
        chainId: 56,
        urls: {
          apiURL: "https://api.bscscan.com/api",
          browserURL: "https://bscscan.com/",
        },
      },
      {
        network: "Matic",
        chainId: 237,
        urls: {
          apiURL: "https://api.polygonscan.com/api",
          browserURL: "https://polygonscan.com/",
        },
      },
      {
        network: "Blast",
        chainId: 81457,
        urls: {
          apiURL: "https://api.blastscan.io/api",
          browserURL: "https://blastscan.io/",
        },
      },
      {
        network: "Base",
        chainId: 8453,
        urls: {
          apiURL: "https://api.basescan.org/api",
          browserURL: "https://basescan.org/",
        },
      },
      {
        network: "zkSync",
        chainId: 324,
        urls: {
          apiURL: "https://api-era.zksync.network/api",
          browserURL: "https://era.zksync.network/",
        },
      },
      {
        network: "Optimism",
        chainId: 10,
        urls: {
          apiURL: "https://api-optimistic.etherscan.io/api",
          browserURL: "https://optimistic.etherscan.io/",
        },
      },
      {
        network: "Arbitrum",
        chainId: 42161,
        urls: {
          apiURL: "https://api.arbiscan.io/api",
          browserURL: "https://arbiscan.io/",
        },
      },
      {
        network: "Linea",
        chainId: 59144,
        urls: {
          apiURL: "https://api.lineascan.build/api",
          browserURL: "https://lineascan.build",
        },
      },
      {
        network: "Scroll",
        chainId: 534352,
        urls: {
          apiURL: "https://api.scrollscan.com/api",
          browserURL: "https://scrollscan.com/",
        },
      },
      {
        network: "Mantle",
        chainId: 5000,
        urls: {
          apiURL: "https://api.mantlescan.xyz/api",
          browserURL: "https://mantlescan.xyz/",
        },
      }
    ],
  },
};
