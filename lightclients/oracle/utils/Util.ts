import { BigNumber, BigNumber } from "ethers";
import { JsonRpcProvider } from "@ethersproject/providers";
const Rpc = require("isomorphic-rpc");
const { encode, toBuffer } = require("eth-util-lite");
import { ethers } from "hardhat";
const Tree = require("merkle-patricia-tree");
const { Header, Proof, Receipt, Transaction } = require("eth-object");
const { promisfy } = require("promisfy");
export class BlockHeader {
    public parentHash?: string;
    public sha3Uncles?: string;
    public miner?: string;
    public stateRoot?: string;
    public transactionsRoot?: string;
    public receiptsRoot?: string;
    public logsBloom?: string;
    public difficulty?: BigNumber;
    public number?: BigNumber;
    public gasLimit?: BigNumber;
    public gasUsed?: BigNumber;
    public timestamp?: BigNumber;
    public extraData: string;
    public mixHash?: string;
    public nonce?: string;
    public baseFeePerGas?: BigNumber;

    constructor(
        parentHash: string,
        sha3Uncles: string,
        miner: string,
        stateRoot: string,
        transactionsRoot: string,
        receiptsRoot: string,
        logsBloom: string,
        difficulty: BigNumber,
        number: BigNumber,
        gasLimit: BigNumber,
        gasUsed: BigNumber,
        timestamp: BigNumber,
        extraData: string,
        mixHash: string,
        nonce: string,
        baseFeePerGas: BigNumber
    ) {
        this.parentHash = parentHash;
        this.sha3Uncles = sha3Uncles;
        this.miner = miner;
        this.stateRoot = stateRoot;
        this.transactionsRoot = transactionsRoot;
        this.receiptsRoot = receiptsRoot;
        this.logsBloom = logsBloom;
        this.difficulty = difficulty;
        (this.number = number), (this.gasLimit = gasLimit), (this.gasUsed = gasUsed);
        this.timestamp = timestamp;
        this.extraData = extraData;
        this.mixHash = mixHash;
        this.nonce = nonce;
        this.baseFeePerGas = baseFeePerGas;
    }
}
// struct TxLog {
//     address addr;
//     bytes[] topics;
//     bytes data;
// }
export class TxLog {
    public addr?: string;
    public topics?: Array<string>;
    public data?: string;

    constructor(addr: string, topics: Array<string>, data: string) {
        this.addr = addr;
        this.topics = topics;
        this.data = data;
    }
}

// struct TxReceipt {
//     uint256 receiptType;
//     bytes postStateOrStatus;
//     uint256 cumulativeGasUsed;
//     bytes bloom;
//     TxLog[] logs;
// }
export class TxReceipt {
    public receiptType?: BigNumber;
    public postStateOrStatus?: string;
    public cumulativeGasUsed?: BigNumber;
    public bloom?: string;
    public logs?: Array<TxLog>;

    constructor(
        receiptType: BigNumber,
        postStateOrStatus: string,
        cumulativeGasUsed: BigNumber,
        bloom: string,
        logs: Array<TxLog>
    ) {
        this.receiptType = receiptType;
        this.postStateOrStatus = postStateOrStatus;
        this.cumulativeGasUsed = cumulativeGasUsed;
        this.bloom = bloom;
        this.logs = logs;
    }
}

// struct ReceiptProof {
//     bytes txReceipt;
//     uint256 receiptType;
//     bytes keyIndex;
//     bytes[] proof;
// }

export class ReceiptProof {
    public txReceipt?: string;
    public receiptType?: BigNumber;
    public keyIndex?: string;
    public proof?: Array<string>;

    constructor(txReceipt: string, receiptType : BigNumber,keyIndex: string, proof: Array<string>) {
        this.txReceipt = txReceipt;
        this.receiptType = receiptType;
        this.keyIndex = keyIndex;
        this.proof = proof;
    }
}

export class ProofData {
    public blockNum?: BigNumber;
    public receiptProof?: ReceiptProof;

    constructor(blockNum: BigNumber, receiptProof: ReceiptProof) {
        this.blockNum = blockNum;
        this.receiptProof = receiptProof;
    }
}


export class ProofStruct {
    public blockNum?: BigNumber;
    public receiptProof?:ReceiptProof;
    public txReceipt?:TxReceipt;
    constructor(blockNum: BigNumber, receiptProof: ReceiptProof,txReceipt: TxReceipt) {
        this.blockNum = blockNum;
        this.receiptProof = receiptProof;
        this.txReceipt = txReceipt;
    }
}

export function index2key(index: number, proofLength: number) {
    const actualkey: Array<number> = new Array<number>();
    const encoded = buffer2hex(encode(index)).slice(2);
    let key = [...new Array(encoded.length / 2).keys()].map((i) => parseInt(encoded[i * 2] + encoded[i * 2 + 1], 16));

    key.forEach((val) => {
        if (actualkey.length + 1 === proofLength) {
            actualkey.push(val);
        } else {
            actualkey.push(Math.floor(val / 16));
            actualkey.push(val % 16);
        }
    });
    return "0x" + actualkey.map((v) => v.toString(16).padStart(2, "0")).join("");
}

function buffer2hex(buffer: Buffer) {
    return "0x" + buffer.toString("hex");
}

export async function getProof(txHash: string, rpc: string) {
    const provider = new ethers.providers.JsonRpcProvider(rpc);

    let r = await provider.getTransactionReceipt(txHash);

    console.log("block =====",r);

    let logs: TxLog[] = new Array<TxLog>();

    for (let i = 0; i < r.logs.length; i++) {
        let log = new TxLog(r.logs[i].address, r.logs[i].topics, r.logs[i].data);

        logs.push(log);
    }
    let txReceipt = new TxReceipt(
        BigNumber.from(r.type),
        BigNumber.from(r.status || r.root).toHexString(),
        BigNumber.from(r.cumulativeGasUsed),
        r.logsBloom,
        logs
    );

    let proof = await getReceipt(txHash, rpc);
    let key = r.transactionIndex === 0 ? "0x0800" : index2key(BigNumber.from(r.transactionIndex).toNumber(), proof.proof.length);
    let receiptProof = new ReceiptProof(
        "",
        BigNumber.from(r.type),
        key,
        proof.proof
    );

    let proofStruct = new ProofStruct(BigNumber.from(r.blockNumber), receiptProof,txReceipt);

    return proofStruct;
}

async function getReceipt(txHash: string, uri?: string) {
    const resp = await receiptProof(txHash, uri);

    let proofs: Array<string> = new Array<string>();

    for (let i = 0; i < resp.receiptProof.length; i++) {
        proofs[i] = "0x" + encode(resp.receiptProof[i]).toString("hex");
    }

    return {
        proof: proofs,
        key: "0x" + encode(Number(resp.txIndex)).toString("hex"), // '0x12' => Nunmber
    };
}

async function receiptProof(txHash: string, uri: string | undefined) {
    let rpc = new Rpc(uri);
    let targetReceipt = await rpc.eth_getTransactionReceipt(txHash);
    if (!targetReceipt) {
        throw new Error("txhash/targetReceipt not found. (use Archive node)");
    }

    let rpcBlock = await rpc.eth_getBlockByHash(targetReceipt.blockHash, false);

    let receipts = await Promise.all(
        rpcBlock.transactions.map((siblingTxHash: string) => {
            return rpc.eth_getTransactionReceipt(siblingTxHash);
        })
    );

    let tree = new Tree();
    await Promise.all(
        receipts.map((siblingReceipt, index) => {
            let siblingPath = encode(index);
            let serializedReceipt = Receipt.fromRpc(siblingReceipt).serialize();
            if (siblingReceipt.type != "0x0") {
                serializedReceipt = Buffer.concat([Buffer.from([siblingReceipt.type]), serializedReceipt]);
            }
            return promisfy(tree.put, tree)(siblingPath, serializedReceipt);
        })
    );

    let [_, __, stack] = await promisfy(tree.findPath, tree)(encode(targetReceipt.transactionIndex));
    return {
        header: Header.fromRpc(rpcBlock),
        receiptProof: Proof.fromStack(stack),
        txIndex: targetReceipt.transactionIndex,
    };
}
