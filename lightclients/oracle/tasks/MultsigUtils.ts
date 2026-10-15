
let quorum: number = 2;
let signers = [
    '0x72F08C970323D3126Bb33d202D9987E0a155b4b2',
    '0x868576F89ABecc902F808F1EE0DE872e1Eb59435',
    '0x04973720Dc478Ba7ed697442398f020B005935b8'
  ]
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
