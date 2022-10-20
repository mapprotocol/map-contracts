const nearAPI = require('near-api-js')
const fs = require("fs");

let rpcUrl = process.env.RPC_URL;

let network = process.env.NETWORK;

async function main() {

    await refreshInitData();
}


async function refreshInitData() {
    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    let near = await nearAPI.connect({
        nodeUrl: rpcUrl,
        deps: {
            keyStore: keyStore
        }
    })

    let last = {
        finality: 'final'
    }

    let lastblock = await near.connection.provider.sendJsonRpc(
        'block',
        last
    )

    last = {
        block_id: lastblock.header.height - 50
    }

    lastblock = await near.connection.provider.sendJsonRpc(
        'block',
        last
    )

    let pre = {
        block_id: lastblock.header.height - 86010
    }

    let preblock = await near.connection.provider.sendJsonRpc(
        'block',
        pre
    )

    let block = await near.connection.provider.sendJsonRpc(
        'next_light_client_block',
        [lastblock.header.hash]
    )

    let validators = await near.connection.provider.sendJsonRpc(
        'next_light_client_block',
        [preblock.header.hash]
    )

    if (block.inner_lite.epoch_id == validators.inner_lite.next_epoch_id) {

        console.log(network.toString().toLowerCase());

        if (network.toString().toLowerCase() == 'testnet') {

            fs.writeFile("./scripts/data/testnet/block.json", JSON.stringify(block), (err) => {
                if (err) {
                    throw new Error(err)
                    return
                }
                console.log("block is write succeed")
            })
            fs.writeFile("./scripts/data/testnet/validators.json", JSON.stringify(validators), (err) => {
                if (err) {
                    throw new Error(err)
                    return
                }
                console.log("validators is write succeed")
            })
        } else {


            fs.writeFile("./scripts/data/mainnet/block.json", JSON.stringify(block), (err) => {
                if (err) {
                    throw new Error(err)
                    return
                }
                console.log("block is write succeed")
            })
            fs.writeFile("./scripts/data/mainnet/validators.json", JSON.stringify(validators), (err) => {
                if (err) {
                    throw new Error(err)
                    return
                }
                console.log("validators is write succeed")
            })

        }

    } else {
        console.log(block.inner_lite.epoch_id)
        console.log(validators.inner_lite.next_epoch_id)
        throw new Error("refresh fail");
    }

}
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});