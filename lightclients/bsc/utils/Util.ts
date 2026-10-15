import { BigNumber} from "ethers";
import { JsonRpcProvider } from "@ethersproject/providers";
import { ethers } from "hardhat";
const Rpc = require("isomorphic-rpc");
const { GetProof } = require("eth-proof");
const { encode } = require("eth-util-lite");
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
        public baseFeePerGas?:BigNumber;
        public withdrawalsRoot?:string;
        public blobGasUsed?:BigNumber;
        public excessBlobGas?:BigNumber;
        public parentBeaconBlockRoot?:string
        public requestsHash ?:string
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
            baseFeePerGas:BigNumber,
            withdrawalsRoot:string,
            blobGasUsed:BigNumber,
            excessBlobGas:BigNumber,
            parentBeaconBlockRoot:string,
            requestsHash:string,
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
            this.withdrawalsRoot = withdrawalsRoot;
            this.blobGasUsed = blobGasUsed;
            this.excessBlobGas = excessBlobGas;
            this.parentBeaconBlockRoot = parentBeaconBlockRoot;
            this.requestsHash = requestsHash;
        }
    }

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


    export class ReceiptProof {
        public txReceipt?: string;
        public keyIndex?: string;
        public proof?: Array<string>;

        constructor(txReceipt: string, keyIndex: string, proof: Array<string>) {
            this.txReceipt = txReceipt;
            this.keyIndex = keyIndex;
            this.proof = proof;
        }
    }

    export async function getBlock(blockNumber: number, provider: JsonRpcProvider) {
        let block = await provider.getBlock(blockNumber);

        const params: { [key: string]: any } = {
            includeTransactions: !!false,
        };
        params.blockHash = block.hash;

        let rpcHeader = await provider.perform("getBlock", params);

        // console.log("rpcHeader ===",rpcHeader)

        let baseFeePerGas = rpcHeader.baseFeePerGas ? BigNumber.from(rpcHeader.baseFeePerGas) : BigNumber.from("0");
        let withdrawalsRoot = rpcHeader.withdrawalsRoot ? rpcHeader.withdrawalsRoot : "0x0000000000000000000000000000000000000000000000000000000000000001";
        let blobGasUsed = rpcHeader.blobGasUsed ? BigNumber.from(rpcHeader.blobGasUsed) : BigNumber.from("0");
        let excessBlobGas = rpcHeader.excessBlobGas ? BigNumber.from(rpcHeader.excessBlobGas) : BigNumber.from("0");
        let parentBeaconBlockRoot = rpcHeader.parentBeaconBlockRoot ? rpcHeader.parentBeaconBlockRoot : "0x0000000000000000000000000000000000000000000000000000000000000001";
        let requestsHash = rpcHeader.requestsHash ? rpcHeader.requestsHash : "0x0000000000000000000000000000000000000000000000000000000000000000";
        let blockHeader = new BlockHeader(
            rpcHeader.parentHash,
            rpcHeader.sha3Uncles,
            rpcHeader.miner,
            rpcHeader.stateRoot,
            rpcHeader.transactionsRoot,
            rpcHeader.receiptsRoot,
            rpcHeader.logsBloom,
            BigNumber.from(rpcHeader.difficulty),
            BigNumber.from(rpcHeader.number),
            BigNumber.from(rpcHeader.gasLimit),
            BigNumber.from(rpcHeader.gasUsed),
            BigNumber.from(rpcHeader.timestamp),
            rpcHeader.extraData,
            rpcHeader.mixHash,
            rpcHeader.nonce,
            baseFeePerGas,
            withdrawalsRoot,
            blobGasUsed,
            excessBlobGas,
            parentBeaconBlockRoot,
            requestsHash
        );
        return blockHeader;
    }

    export async function getTxReceipt(txHash: string, rpc: string = "") {
        const provider = new ethers.providers.JsonRpcProvider(rpc);

        let r = await provider.getTransactionReceipt(txHash);

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
        return txReceipt;
    }

    export async  function getReceiptProof(txHash: string, uri?: string) {
        const resp = await receiptProof(txHash, uri);

        let proofs: Array<string> = new Array<string>();

        for (let i = 0; i < resp.receiptProof.length; i++) {
            proofs[i] = "0x" + encode(resp.receiptProof[i]).toString("hex");
        }
        let key  = resp.txIndex === 0 ? 0x0800 : index2key(BigNumber.from(resp.txIndex).toNumber(), proofs.length);
        return {
            proof: proofs,
            key: key
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

    function index2key(index: number, proofLength: number) {
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