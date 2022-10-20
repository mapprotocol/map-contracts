require('@nomiclabs/hardhat-waffle')
require('hardhat-gas-reporter')
require('hardhat-spdx-license-identifier')
require('hardhat-deploy')
require('hardhat-abi-exporter')
require('@nomiclabs/hardhat-ethers')
require('dotenv/config')
require('@nomiclabs/hardhat-etherscan')
require('solidity-coverage')


const { PRIVATE_KEY, INFURA_KEY} = process.env;


let accounts = [];
// task("accounts", "Prints the list of accounts", async (taskArgs, hre) => {
//   const accounts = await hre.ethers.getSigners();
//
//   for (const account of accounts) {
//     console.log(account.address);
//   }
// });
accounts.push(PRIVATE_KEY);

var fs = require("fs");
var read = require('read');
var util = require('util');
const keythereum = require("keythereum");
const prompt = require('prompt-sync')();
(async function() {
  try {
    const root = '.keystore';
    var pa = fs.readdirSync(root);
    for (let index = 0; index < pa.length; index ++) {
      let ele = pa[index];
      let fullPath = root + '/' + ele;
      var info = fs.statSync(fullPath);
      //console.dir(ele);
      if(!info.isDirectory() && ele.endsWith(".keystore")){
        const content = fs.readFileSync(fullPath, 'utf8');
        const json = JSON.parse(content);
        const password = prompt('Input password for 0x' + json.address + ': ', {echo: '*'});
        //console.dir(password);
        const privatekey = keythereum.recover(password, json).toString('hex');
        //console.dir(privatekey);
        accounts.push('0x' + privatekey);
        //console.dir(keystore);
      }
    }
  } catch (ex) {
  }
  try {
    const file = '.secret';
    var info = fs.statSync(file);
    if (!info.isDirectory()) {
      const content = fs.readFileSync(file, 'utf8');
      let lines = content.split('\n');
      for (let index = 0; index < lines.length; index ++) {
        let line = lines[index];
        if (line == undefined || line == '') {
          continue;
        }
        if (!line.startsWith('0x') || !line.startsWith('0x')) {
          line = '0x' + line;
        }
        accounts.push(line);
      }
    }
  } catch (ex) {
  }
})();


module.exports = {
  defaultNetwork: 'hardhat',
  abiExporter: {
    path: './abi',
    clear: false,
    flat: true,
  },
  namedAccounts: {
    deployer: {
      default: 0,
      0: '0x289F8F063c4304F432bb96DD31e82bdCc5CcE142',
      1: '0x038BCF8d2d48C084B661E3f2B3c514b4244B4D90',
      3: '0x289F8F063c4304F432bb96DD31e82bdCc5CcE142',
      56: '0x038BCF8d2d48C084B661E3f2B3c514b4244B4D90',
      97: '0x289F8F063c4304F432bb96DD31e82bdCc5CcE142',
      137:'0x038BCF8d2d48C084B661E3f2B3c514b4244B4D90',
      22776: '0x289F8F063c4304F432bb96DD31e82bdCc5CcE142',
    },
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
      chainId:212
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
      url: `http://3.0.19.66:7445`,
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
    Eth: {
      url: `https://mainnet.infura.io/v3/` + INFURA_KEY,
      chainId : 1,
      accounts: accounts
    },
    ETHP: {
      url: `http://18.138.248.113:8545`,
      chainId : 34434,
      accounts: accounts
    },
    Sepolia: {
      url: `https://rpc.sepolia.org`,
      chainId : 11155111,
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
