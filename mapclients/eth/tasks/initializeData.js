let fs = require("fs");
let request = require('request');

function httprequest(url,requestData){
    return new Promise((resolve, reject)=>{

        let option ={
            url: url.toString(),
            method: "POST",
            json: true,
            headers: {
                "content-type": "application/json",
            },
            body: requestData
        }
        request(option, function(error, response, body) {
            resolve(body)
        });
    });

}


module.exports = async (taskArgs,hre) => {
    let requestData={
        "method":"istanbul_getEpochInfo",
        "params":[Number(taskArgs.epoch)],
        "id":1
    }
    let data = await httprequest(hre.network.config.url,requestData);

    let datar = "let initData =" + JSON.stringify(data.result) + "\n" + "module.exports = initData"


    fs.writeFileSync('./deploy/config.js', datar);

    console.log(`write in epco  ${data.result.epoch} success`)
}