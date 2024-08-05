
let quorum : number = 1

let signers = ["0xE796bc0Ef665D5F730408a55AA0FF4e6f8B90920","0xE0DC8D7f134d0A79019BEF9C2fd4b2013a64fCD6"]

export class Multisig {
    public quorum?: number;
    public signers?: Array<string>;
    
    constructor(
        quorum:number = 0,
        signers:Array<string> = ["0x"]
    ) {
        this.quorum = quorum;
        this.signers = signers;
    }
}

export async  function compare(version:string = "0x",multisig:Multisig):Promise<Boolean>{
    let p = ethers.utils.solidityPack(["uint256","address[]"],[multisig.quorum,multisig.signers]);

    let v = await ethers.utils.keccak256(p);

   return version == v;
}

export function getSigInfo():Multisig{
    
    let m = new Multisig(quorum,signers)

   return m;
}