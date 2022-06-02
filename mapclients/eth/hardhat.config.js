require('@nomiclabs/hardhat-waffle')
require('hardhat-gas-reporter')
require('hardhat-spdx-license-identifier')
require('hardhat-deploy')
require('hardhat-abi-exporter')
require('@nomiclabs/hardhat-ethers')
require('dotenv/config')
require('@nomiclabs/hardhat-etherscan')
require('solidity-coverage')


const { PRIVATE_KEY, ETH_INFURA_KEY, INFURA_KEY, HECO_SCAN_KEY} = process.env;


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
    wcoin: {
      default: 0,
      1: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
      3: '0xf70949bc9b52deffcda63b0d15608d601e3a7c49',
      56: '0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c',
      97: '0xf984Ad9299B0102426a646aF72e2052a3A7eD0E2',
      22776: '0x13cb04d4a5dfb6398fc5ab005a6c84337256ee23',
      137: '0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270',
    },
    mapcoin: {
      default: 0,
      1: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
      3: '0x47f423C44976Fbe745588020b85B09A56458f9C0',
      56: '0x8105ECe4ce08B6B6449539A5db23e23b973DfA8f',
      97: '0x624F96Ea37bBbEA15Df489f9083Fe786BAf15723',
      22776: '0x0000000000000000000000000000000000000000',
      137: '0x659BC6aD25AEea579f3eA91086fDbc7ac0432Dc4',
    },
    usdt: {
      default: 0,
      22776: '0x33daba9618a75a7aff103e53afe530fbacf4a3dd',
    },
    usdc: {
      default: 0,
      22776: '0x9f722b2cb30093f766221fd0d37964949ed66918',
    },
    eth: {
      default: 0,
      22776: '0x05ab928d446d8ce6761e368c8e7be03c3168a9ec',
    },
    idv: {
      default: 0,
      22776: '0xeac6cfd6e9e2fa033d85b7abdb6b14fe8aa71f2a',
    }
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
    bscmain: {
      url: `https://bsc-dataseed2.defibit.io/`,
      accounts: accounts,
      chainId: 56,
      gasMultiplier: 1.5,
      gasPrice: 5.5 * 1000000000
    },
    MaticTest: {
      url: `https://rpc-mumbai.maticvigil.com/`,
      chainId : 80001,
      accounts: accounts
    },
    Matic: {
      url: `https://rpc-mainnet.maticvigil.com`,
      chainId : 137,
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
    Eth: {
      url: `https://mainnet.infura.io/v3/` + ETH_INFURA_KEY,
      chainId : 1,
      accounts: accounts
    },
    Ropsten: {
      url: `https://ropsten.infura.io/v3/` + INFURA_KEY,
      chainId : 3,
      accounts: accounts
    },
    Map: {
      url: `https://poc2-rpc.maplabs.io`,
      chainId : 22776,
      accounts: accounts
    },
    Map2: {
      url: `http://18.142.54.137:7445`,
      chainId : 29088,
      accounts: accounts
    },
    Bsc: {
      url: `https://bsc-dataseed1.binance.org/`,
      chainId : 56,
      accounts: accounts
    },
    BscTest: {
      url: `https://data-seed-prebsc-2-s3.binance.org:8545/`,
      chainId : 97,
      accounts: accounts,
      gasPrice: 11 * 1000000000
    },
    BscTest2: {
      url: `https://data-seed-prebsc-2-s2.binance.org:8545/`,
      chainId : 97,
      accounts: accounts,
      gasPrice: 11 * 1000000000
    },
    Abey: {
      url: `http://54.169.112.1:8545`,
      chainId : 179,
      accounts: accounts
    },
    True: {
      url: `https://rpc.truescan.network/`,
      chainId : 19330,
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
