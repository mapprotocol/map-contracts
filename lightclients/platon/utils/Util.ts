import { BigNumber } from "ethers";
import { JsonRpcProvider, BaseProvider } from "@ethersproject/providers";
const Rpc = require("isomorphic-rpc");
const { encode, toBuffer } = require("eth-util-lite");
import { ethers } from "hardhat";
const Tree = require("merkle-patricia-tree");
const { Header, Proof, Receipt, Transaction } = require("eth-object");
const { promisfy } = require("promisfy");
import { req } from "../utils/httpUtil";
export class BlockHeader {
    public parentHash?: string;
    public miner?: string;
    public stateRoot?: string;
    public transactionsRoot?: string;
    public receiptsRoot?: string;
    public logsBloom?: string;
    public number?: BigNumber;
    public gasLimit?: BigNumber;
    public gasUsed?: BigNumber;
    public timestamp?: BigNumber;
    public extraData: string;
    public nonce?: string;

    constructor(
        parentHash: string,
        miner: string,
        stateRoot: string,
        transactionsRoot: string,
        receiptsRoot: string,
        logsBloom: string,
        number: BigNumber,
        gasLimit: BigNumber,
        gasUsed: BigNumber,
        timestamp: BigNumber,
        extraData: string,
        nonce: string
    ) {
        this.parentHash = parentHash;
        this.miner = miner;
        this.stateRoot = stateRoot;
        this.transactionsRoot = transactionsRoot;
        this.receiptsRoot = receiptsRoot;
        this.logsBloom = logsBloom;
        (this.number = number), (this.gasLimit = gasLimit), (this.gasUsed = gasUsed);
        this.timestamp = timestamp;
        this.extraData = extraData;
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

//   struct ReceiptProof {
//     TxReceipt txReceipt;
//     bytes keyIndex;
//     bytes[] proof;
// }

export class ReceiptProof {
    public txReceipt?: TxReceipt;
    public keyIndex?: string;
    public proof?: Array<string>;

    constructor(txReceipt: TxReceipt, keyIndex: string, proof: Array<string>) {
        this.txReceipt = txReceipt;
        this.keyIndex = keyIndex;
        this.proof = proof;
    }
}
// struct QuorumCert {
//     uint256 epoch;
//     uint256 viewNumber;
//     bytes32 blockHash;
//     uint256 blockNumber;
//     uint256 blockIndex;
//     bytes signature;
//     uint256[] validatorSet;
// }
export class QuorumCert {
    public epoch?: BigNumber;
    public viewNumber?: BigNumber;
    public blockHash?: string;
    public blockNumber?: BigNumber;
    public blockIndex?: BigNumber;
    public signature?: string;
    public validatorSignBitMap?: BigNumber;
    public signedCount?: number;

    constructor(
        epoch: BigNumber,
        viewNumber: BigNumber,
        blockHash: string,
        blockNumber: BigNumber,
        blockIndex: BigNumber,
        signature: string,
        validatorSignBitMap: BigNumber,
        signedCount: number
    ) {
        this.epoch = epoch;
        this.viewNumber = viewNumber;
        this.blockHash = blockHash;
        this.blockNumber = blockNumber;
        this.blockIndex = blockIndex;
        this.signature = signature;
        this.validatorSignBitMap = validatorSignBitMap;
        this.signedCount = signedCount;
    }
}
export class ProofData {
    public header?: BlockHeader;
    public receiptProof?: ReceiptProof;
    public quorumCert?: QuorumCert;

    constructor(header: BlockHeader, receiptProof: ReceiptProof, quorumCert: QuorumCert) {
        this.header = header;
        this.receiptProof = receiptProof;
        this.quorumCert = quorumCert;
    }
}

export async function getBlock(blockNumber: number, provider: BaseProvider) {
    let block = await provider.getBlock(blockNumber);

    const params: { [key: string]: any } = {
        includeTransactions: !!false,
    };
    params.blockHash = block.hash;

    let rpcHeader = await provider.perform("getBlock", params);

    let blockHeader = new BlockHeader(
        rpcHeader.parentHash,
        rpcHeader.miner,
        rpcHeader.stateRoot,
        rpcHeader.transactionsRoot,
        rpcHeader.receiptsRoot,
        rpcHeader.logsBloom,
        BigNumber.from(rpcHeader.number),
        BigNumber.from(rpcHeader.gasLimit),
        BigNumber.from(rpcHeader.gasUsed),
        BigNumber.from(rpcHeader.timestamp),
        rpcHeader.extraData,
        rpcHeader.nonce
    );

    return blockHeader;
}

export async function getQuorumCert(hash: string, rpc: string) {
    let methons = "platon_getBlockQuorumCertByHash";
    let params: any[] = [[hash]];
    let data = await req(rpc, methons, params);
    let q = data.result[0];
    let signedCount = 0;
    let validatorSignBitMap: BigNumber = BigNumber.from("0");

    for (let i = 0; i < q.validatorSet.length; i++) {
        if (q.validatorSet[i] === "x" || q.validatorSet[i] === "X") {
            signedCount++;
            validatorSignBitMap = validatorSignBitMap.add(BigNumber.from("1").shl(i));
        }
    }

    let quorumCert = new QuorumCert(
        q.epoch,
        q.viewNumber,
        q.blockHash,
        q.blockNumber,
        q.blockIndex,
        q.signature,
        validatorSignBitMap,
        signedCount
    );

    return quorumCert;
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

    console.log(r.blockNumber);

    let block = await getBlock(r.blockNumber, provider);

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

    let receiptProof = new ReceiptProof(
        txReceipt,
        index2key(BigNumber.from(r.transactionIndex).toNumber(), proof.proof.length),
        proof.proof
    );

    let quorumCert = await getQuorumCert(txHash, rpc);

    let proofData = new ProofData(block, receiptProof, quorumCert);

    return proofData;
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
