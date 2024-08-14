
// let quorum: number = 1;
// let signers = ["0xE796bc0Ef665D5F730408a55AA0FF4e6f8B90920", "0xE0DC8D7f134d0A79019BEF9C2fd4b2013a64fCD6"];

// let quorum: number = 2;
// let signers = ["0x72f08c970323d3126bb33d202d9987e0a155b4b2", "0xE0DC8D7f134d0A79019BEF9C2fd4b2013a64fCD6", "0x39873f8B348c2990aE1E4435896C154a87f84b6e", "0x30Ef5A2483B6852174AC149c521bA8Ae9F843Dc4"];

let quorum: number = 1;
let signers = ["0x386ce1a187eC7329CFb8E467EB02FB07c698256A", "0xE0DC8D7f134d0A79019BEF9C2fd4b2013a64fCD6"];

export class Multisig {
    public quorum?: number;
    public signers?: Array<string>;

    constructor(quorum: number = 0, signers: Array<string> = ["0x"]) {
        this.quorum = quorum;
        this.signers = signers;
    }
}

export async function compare(version: string = "0x", multisig: Multisig): Promise<Boolean> {
    let p = ethers.utils.solidityPack(["uint256", "address[]"], [multisig.quorum, multisig.signers]);

    let v = await ethers.utils.keccak256(p);

    return version == v;
}

export function getSigInfo(): Multisig {
    let m = new Multisig(quorum, signers);

    return m;
}
