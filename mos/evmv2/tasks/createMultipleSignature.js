module.exports = async (taskArgs, hre) => {

    const {deploy} = hre.deployments
    const accounts = await ethers.getSigners()
    const deployer = accounts[0];

    let multipleAddress = taskArgs.multiuser.split(",")
    console.log(multipleAddress);

    let gnosisSafeFactoryAbi = [
        {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": false,
                    "internalType": "contract GnosisSafeProxy",
                    "name": "proxy",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "address",
                    "name": "singleton",
                    "type": "address"
                }
            ],
            "name": "ProxyCreation",
            "type": "event"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "_singleton",
                    "type": "address"
                },
                {
                    "internalType": "bytes",
                    "name": "initializer",
                    "type": "bytes"
                },
                {
                    "internalType": "uint256",
                    "name": "saltNonce",
                    "type": "uint256"
                }
            ],
            "name": "calculateCreateProxyWithNonceAddress",
            "outputs": [
                {
                    "internalType": "contract GnosisSafeProxy",
                    "name": "proxy",
                    "type": "address"
                }
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "singleton",
                    "type": "address"
                },
                {
                    "internalType": "bytes",
                    "name": "data",
                    "type": "bytes"
                }
            ],
            "name": "createProxy",
            "outputs": [
                {
                    "internalType": "contract GnosisSafeProxy",
                    "name": "proxy",
                    "type": "address"
                }
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "_singleton",
                    "type": "address"
                },
                {
                    "internalType": "bytes",
                    "name": "initializer",
                    "type": "bytes"
                },
                {
                    "internalType": "uint256",
                    "name": "saltNonce",
                    "type": "uint256"
                },
                {
                    "internalType": "contract IProxyCreationCallback",
                    "name": "callback",
                    "type": "address"
                }
            ],
            "name": "createProxyWithCallback",
            "outputs": [
                {
                    "internalType": "contract GnosisSafeProxy",
                    "name": "proxy",
                    "type": "address"
                }
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address",
                    "name": "_singleton",
                    "type": "address"
                },
                {
                    "internalType": "bytes",
                    "name": "initializer",
                    "type": "bytes"
                },
                {
                    "internalType": "uint256",
                    "name": "saltNonce",
                    "type": "uint256"
                }
            ],
            "name": "createProxyWithNonce",
            "outputs": [
                {
                    "internalType": "contract GnosisSafeProxy",
                    "name": "proxy",
                    "type": "address"
                }
            ],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "proxyCreationCode",
            "outputs": [
                {
                    "internalType": "bytes",
                    "name": "",
                    "type": "bytes"
                }
            ],
            "stateMutability": "pure",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "proxyRuntimeCode",
            "outputs": [
                {
                    "internalType": "bytes",
                    "name": "",
                    "type": "bytes"
                }
            ],
            "stateMutability": "pure",
            "type": "function"
        },
        {
            "inputs": [
                {
                    "internalType": "address[]",
                    "name": "_owners",
                    "type": "address[]"
                },
                {
                    "internalType": "uint256",
                    "name": "_threshold",
                    "type": "uint256"
                },
                {
                    "internalType": "address",
                    "name": "to",
                    "type": "address"
                },
                {
                    "internalType": "bytes",
                    "name": "data",
                    "type": "bytes"
                },
                {
                    "internalType": "address",
                    "name": "fallbackHandler",
                    "type": "address"
                },
                {
                    "internalType": "address",
                    "name": "paymentToken",
                    "type": "address"
                },
                {
                    "internalType": "uint256",
                    "name": "payment",
                    "type": "uint256"
                },
                {
                    "internalType": "address payable",
                    "name": "paymentReceiver",
                    "type": "address"
                }
            ],
            "name": "setup",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ];
    let safeFactory = await ethers.getContractAt(gnosisSafeFactoryAbi,taskArgs.safeaddress);

    let factoryData = safeFactory.interface.encodeFunctionData(
        "setup",[
            multipleAddress,
            taskArgs.threshold,
            "0x0000000000000000000000000000000000000000",
            "0x",
            "0xf48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4",
            "0x0000000000000000000000000000000000000000",
            "0",
            "0x0000000000000000000000000000000000000000"
        ]
    )

    await (await safeFactory.connect(deployer).createProxyWithNonce(
        "3e5c63644e683549055b9be8653de26e0b4cd36e",
        factoryData,
        taskArgs.saltnonce,
        {
            gasLimit:"20000000"
        }
    )).wait();

    async function calculateProxyAddress (factory, singleton, inititalizer, nonce) {
        const deploymentCode = ethers.utils.solidityPack(["bytes", "uint256"], [await factory.proxyCreationCode(), singleton])
        const salt = ethers.utils.solidityKeccak256(
            ["bytes32", "uint256"],
            [ethers.utils.solidityKeccak256(["bytes"], [inititalizer]), nonce]
        )
        return ethers.utils.getCreate2Address(factory.address, salt, ethers.utils.keccak256(deploymentCode))
    }

    const gnosisSafeValut =await calculateProxyAddress(
        safeFactory,
        "0x3e5c63644e683549055b9be8653de26e0b4cd36e",
        factoryData,
        taskArgs.saltnonce
    );

    console.log("Gnosis Safe multiple signatures address: ",gnosisSafeValut)


}