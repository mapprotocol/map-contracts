let fs = require("fs");
let request = require("request");

function httprequest(url, requestData) {
    return new Promise((resolve, reject) => {
        let option = {
            url: url.toString(),
            method: "POST",
            json: true,
            headers: {
                "content-type": "application/json",
            },
            body: requestData,
        };
        ``;
        request(option, function (error, response, body) {
            resolve(body);
        });
    });
}

module.exports = async (taskArgs, hre) => {
    let requestData = {
        method: "istanbul_getEpochInfo",
        params: [Number(taskArgs.epoch)],
        id: 1,
    };

    let data = await httprequest(hre.network.config.url, requestData);
    let EpochInfo = data.result;
    console.log(EpochInfo);

    let weights = [];
    for (let i = 0; i < EpochInfo.validators.length; i++) {
        let addLenght =
            "0x000000000000000000000000000000000000000000000000000000000000000" + EpochInfo.validators[i].weight;

        weights.push(addLenght);
    }

    let blockNumber = "0x" + (Number(EpochInfo.epoch) * Number(EpochInfo.epoch_size)).toString(16);

    let getValidatorsBLSPublicKeys = {
        method: "istanbul_getValidatorsBLSPublicKeys",
        params: [blockNumber],
        id: 1,
    };

    let blsData = await httprequest(hre.network.config.url, getValidatorsBLSPublicKeys);
    let G2Hex = blsData.result;
    let vCount = G2Hex.length;
    let maxValidatorsLength = 32 * 5 * 128;
    let initValidatorsInfo = "0x" + G2Hex.map((item, index) => item.slice(2) + weights[index].slice(2)).join("");
    let padLength = maxValidatorsLength - G2Hex.length * 32 * 5;

    initValidatorsInfo = initValidatorsInfo + "00".repeat(padLength);

    let initdata = {
        epoch: EpochInfo.epoch,
        epochSize: EpochInfo.epoch_size,
        threshold: EpochInfo.threshold,
        weights: weights,
        validatorsCount: vCount,
        g2hex: G2Hex,
        validatorsInfo: initValidatorsInfo,
    };
    console.log(initdata);

    let datar;
    if (hre.network.config.chainId == 22776) {
        datar = "let initData =" + JSON.stringify(initdata) + "\n" + "module.exports = initData";
        fs.writeFileSync("./deploy/config.mainnet.js", datar);
        console.log(`write epoch ${data.result.epoch} in mainnet success`);
    } else {
        datar = "let initData =" + JSON.stringify(initdata) + "\n" + "module.exports = initData";
        fs.writeFileSync("./deploy/config.testnet.js", datar);
        console.log(`write epoch ${data.result.epoch} in test success`);
    }

    console.log(`write in epoch  ${data.result.epoch} success`);
};
