let request = require("request");

function httprequest(url: string, requestData: any) {
    return new Promise((resolve, reject) => {
        let option = {
            url: url.toString(),
            method: "POST",
            timeout: 5000,
            // proxy: 'http://127.0.0.1:7890',
            json: true,
            headers: {
                "content-type": "application/json",
            },
            body: requestData,
        };
        request(option, function (error: any, response: any, body: unknown) {
            resolve(body);
        });
    });
}

export async function req(url: string, method: string, params: any[]) {
    let requestData = {
        jsonrpc: "2.0",
        method: method,
        params: params,
        id: 1,
    };
    let data = await httprequest(url, requestData);

    return data;
}
