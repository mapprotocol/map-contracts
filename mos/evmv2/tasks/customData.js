const { BigNumber} = require("ethers");
let fs = require("fs");
let path = require("path");

function getStat(path){
    return new Promise((resolve, reject) => {
        fs.stat(path, (err, stats) => {
            if(err){
                resolve(false);
            }else{
                resolve(stats);
            }
        })
    })
}

function mkdir(dir){
    return new Promise((resolve, reject) => {
        fs.mkdir(dir, err => {
            if(err){
                resolve(false);
            }else{
                resolve(true);
            }
        })
    })
}


async function dirExists(dir){
    let isExists = await getStat(dir);

    if(isExists && isExists.isDirectory()){
        return true;
    }else if(isExists){
        return false;
    }

    let tempDir = path.parse(dir).dir;

    let status = await dirExists(tempDir);
    let mkdirStatus;
    if(status){
        mkdirStatus = await mkdir(dir);
    }
    return mkdirStatus;
}

module.exports = async (taskArgs, hre) => {

    let gnosisABI =
        [
            {
                "inputs": [],
                "name": "nonce",
                "outputs": [
                    {
                        "internalType": "uint256",
                        "name": "",
                        "type": "uint256"
                    }
                ],
                "stateMutability": "view",
                "type": "function"
            },
        ]

    let mos;

    if(taskArgs.ctype === "relay"){
        mos = await ethers.getContractAt('MAPOmnichainServiceRelayV2',taskArgs.targetaddress);
    } else if(taskArgs.ctype === "mos") {
        mos = await ethers.getContractAt('MAPOmnichainServiceV2',taskArgs.targetaddress);
    } else if(taskArgs.ctype === "register") {
        mos = await ethers.getContractAt('TokenRegisterV2',taskArgs.targetaddress);
    }



    let timeLock = await ethers.getContractAt('TimelockController',taskArgs.timelockaddress);


    let valueData = taskArgs.methodarg.split(",")

    let mosData = mos.interface.encodeFunctionData(taskArgs.method, valueData);
    console.log("mosData:",mosData);

    let safe = await ethers.getContractAt(gnosisABI,taskArgs.safeaddress);

    let safeNonce = await safe.nonce();

    let  num = ethers.utils.formatUnits(safeNonce,0)
    console.log("Gnosis Safe Nonce:",num)
    let salt = await ethers.utils.keccak256(ethers.utils.toUtf8Bytes(num))

    console.log("Time Lock Salt:",salt)

    let timeLockData = timeLock.interface.encodeFunctionData("schedule",[
        taskArgs.targetaddress,
        taskArgs.valuenum,
        mosData,
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        salt,
        taskArgs.delaynum
    ])
    console.log("timeLockData:",timeLockData)

    let scheduleData =
        {
            target:taskArgs.targetaddress,
            value:taskArgs.valuenum,
            data:mosData,
            predecessor:"0x0000000000000000000000000000000000000000000000000000000000000000",
            salt:salt,
            delay:taskArgs.delaynum,
            timeLockData:timeLockData
        }

    let basePath = "./log/" + hre.network.config.chainId

    await dirExists(basePath);

    let path =  basePath +"/" + "gnosisSafe" + num + ".js"

    let  datar = "let gnosisSafe" + num + "Data =" + JSON.stringify(scheduleData) + "\n" + "module.exports = " + "gnosisSafe" + num + "Data"

    fs.writeFileSync(path, datar);

    console.log(`write ${path} file success` )

}


