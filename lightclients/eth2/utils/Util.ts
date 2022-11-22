import {BigNumber} from "ethers";
import {JsonRpcProvider} from "@ethersproject/providers";
import {keccak256} from "ethers/lib/utils";

const Rpc = require('isomorphic-rpc')
const {GetProof} = require('eth-proof')
const {encode} = require('eth-util-lite')


export class BeaconBlockHeader {

    public slot?: BigNumber
    public proposerIndex?: BigNumber
    public parentRoot?: string;
    public stateRoot?: string;
    public bodyRoot?: string

    constructor(slot: BigNumber,
                proposerIndex: BigNumber,
                parentRoot: string,
                stateRoot: string,
                bodyRoot: string) {

        this.slot = slot;
        this.proposerIndex = proposerIndex;
        this.parentRoot = parentRoot;
        this.stateRoot = stateRoot;
        this.bodyRoot = bodyRoot;
    }

}

export class BlockHeader {

    // struct BlockHeader {
    //     bytes parentHash;
    //     bytes sha3Uncles;
    //     address miner;
    //     bytes stateRoot;
    //     bytes transactionsRoot;
    //     bytes receiptsRoot;
    //     bytes logsBloom;
    //     uint256 difficulty;
    //     uint256 number;
    //     uint256 gasLimit;
    //     uint256 gasUsed;
    //     uint256 timestamp;
    //     bytes extraData;
    //     bytes mixHash;
    //     bytes nonce;
    // }
    public parentHash?: string;
    public sha3Uncles?: string;
    public miner?: string;
    public stateRoot?: string;
    public transactionsRoot?: string
    public receiptsRoot?: string
    public logsBloom?: string
    public difficulty?: BigNumber
    public number?: BigNumber
    public gasLimit?: BigNumber
    public gasUsed?: BigNumber
    public timestamp?: BigNumber
    public extraData: string
    public mixHash?: string
    public nonce?: string

    constructor(parentHash: string,
        sha3Uncles: string,
        miner: string, stateRoot: string,
        transactionsRoot: string,
        receiptsRoot: string,
        logsBloom: string,
        difficulty: BigNumber,
        number: BigNumber,
        gasLimit: BigNumber,
        gasUsed: BigNumber,
        timestamp: BigNumber,
        extraData: string,
        mixHash: string, nonce: string) {

        this.parentHash = parentHash;
        this.sha3Uncles = sha3Uncles;
        this.miner = miner;
        this.stateRoot = stateRoot;
        this.transactionsRoot = transactionsRoot;
        this.receiptsRoot = receiptsRoot;
        this.logsBloom = logsBloom;
        this.difficulty = difficulty;
        this.number = number,
            this.gasLimit = gasLimit,
            this.gasUsed = gasUsed;
        this.timestamp = timestamp;
        this.extraData = extraData;
        this.mixHash = mixHash;
        this.nonce = nonce;

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
    public data?: string

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
    public logs?: Array<TxLog>

    constructor(receiptType: BigNumber, postStateOrStatus: string, cumulativeGasUsed: BigNumber, bloom: string, logs: Array<TxLog>) {
        this.receiptType = receiptType;
        this.postStateOrStatus = postStateOrStatus;
        this.cumulativeGasUsed = cumulativeGasUsed;
        this.bloom = bloom;
        this.logs = logs;
    }
}


//   struct ReceiptProof {
//     TxReceipt txReceipt;
//     bytes keyIndex;
//     bytes[] proof;
// }

export class ReceiptProof {
    public header?: BlockHeader;
    public txReceipt?: TxReceipt;
    public keyIndex?: string;
    public proof?: Array<string>;

    constructor(header: BlockHeader, txReceipt: TxReceipt, keyIndex: string, proof: Array<string>) {
        this.header = header;
        this.txReceipt = txReceipt;
        this.keyIndex = keyIndex;
        this.proof = proof
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

export class DProofData {
    public headers?: Array<BlockHeader>;
    public receiptProof?: ReceiptProof;


    constructor(headers: Array<BlockHeader>, receiptProof: ReceiptProof) {
        this.headers = headers;
        this.receiptProof = receiptProof;

    }
}

export async function getBlock(blockNumber: number, provider: JsonRpcProvider) {

    let block = await provider.getBlock(blockNumber);


    const params: { [key: string]: any } = {
        includeTransactions: !!false
    };
    params.blockHash = block.hash;

    let rpcHeader = await provider.perform("getBlock", params);

    let blockHeader = new BlockHeader(rpcHeader.parentHash, rpcHeader.sha3Uncles,
        rpcHeader.miner, rpcHeader.stateRoot, rpcHeader.transactionsRoot,
        rpcHeader.receiptsRoot, rpcHeader.logsBloom, BigNumber.from(rpcHeader.difficulty), BigNumber.from(rpcHeader.number),
        BigNumber.from(rpcHeader.gasLimit), BigNumber.from(rpcHeader.gasUsed), BigNumber.from(rpcHeader.timestamp),
        rpcHeader.extraData, rpcHeader.mixHash, rpcHeader.nonce);

    return blockHeader;
}

export function index2key(index: number, proofLength: number) {
    const actualkey: Array<number> = new Array<number>;
    const encoded = buffer2hex(encode(index)).slice(2);
    let key = [...new Array(encoded.length / 2).keys()].map(i => parseInt(encoded[i * 2] + encoded[i * 2 + 1], 16));

    key.forEach(val => {
        if (actualkey.length + 1 === proofLength) {
            actualkey.push(val);
        } else {
            actualkey.push(Math.floor(val / 16));
            actualkey.push(val % 16);
        }
    });
    return '0x' + actualkey.map(v => v.toString(16).padStart(2, '0')).join('');
}

function buffer2hex(buffer: Buffer) {
    return '0x' + buffer.toString('hex');
}

export async function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export function getPubkeySlice(pubkeys: Uint8Array[], start: number, end: number): Uint8Array {
    let pubkeysSlice: Uint8Array = new Uint8Array(48 * (end - start));

    for (let i = start, j = 0; i < end; i++, j++) {
        pubkeysSlice.set(pubkeys[i], 48 * j)
    }

    return pubkeysSlice
}

export function getPubkeyStringSlice(pubkeys: string[], start: number, end: number): Uint8Array {
    let pubkeysSlice: Uint8Array = new Uint8Array(48 * (end - start));

    for (let i = start, j = 0; i < end; i++, j++) {
        pubkeysSlice.set(hexToBytes(pubkeys[i]), 48 * j)
    }

    return pubkeysSlice
}

function hexToBytes(hex: string) : number[]{
    let start = 0
    if (hex.startsWith("0x")) {
        start = 2
    }
    let bytes = [];
    for (let c = start; c < hex.length; c += 2) {
        bytes.push(parseInt(hex.substring(c, c + 2), 16));
    }
    return bytes;
}

export function computePubkeyHash(pubkeys: Uint8Array[], start: number, end: number): string {
    let pubkeysSlice: Uint8Array = new Uint8Array(48 * (end - start));

    for (let i = start, j = 0; i < end; i++, j++) {
        pubkeysSlice.set(pubkeys[i], 48 * j)
    }

    return keccak256(pubkeysSlice)
}