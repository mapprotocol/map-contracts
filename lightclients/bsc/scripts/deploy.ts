import { BigNumber ,Contract} from "ethers";
import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { ethers } from "hardhat";
import { BlockHeader, getBlock, getTxReceipt, getReceiptProof} from "../utils/Util";

let uri = process.env.BSCURI;
let minEpochBlockExtraDataLen = process.env.MinEpochBlockExtraDataLen;
let chainId = process.env.CHAINID;
let epochNum = 200;

async function main() {
    let [wallet] = await ethers.getSigners();
    console.log("begin ...");

    // await updateHeaderV2()
    await verifyV2()
    // await rlpDecodeVoteData(43738000)
}

async function rlpDecodeVoteData(block:number) {
    const provider = new ethers.providers.JsonRpcProvider(uri);
    let header = await getBlock(block , provider);
    let extraData = header.extraData;
    let data;
    if(block % epochNum == 0) {
        let num = BigNumber.from(extraData.substring(66,68)).toNumber();
        console.log(num)
        let validatorsLen = num * (136);
        // before let start = 66 + 2 + validatorsLen;
        // After bohr fork 
        let start = 66 + 2 + validatorsLen + 2;
        data = "0x" + extraData.substring(start, extraData.length - 130);
    } else {
        data = "0x" + extraData.substring(66,extraData.length - 130);
    }
    console.log(data);
    let result = ethers.utils.RLP.decode(data);
    let VoteAttestation = {
        VoteAddressSet : BigNumber.from(result[0]).toNumber(),
        Signature: result[1],
        VoteData : {
            SourceNumber: BigNumber.from(result[2][0]),
            SourceHash: result[2][1],
            TargetNumber: BigNumber.from(result[2][2]),
            TargetHash: result[2][3]
        }
    }
    console.log(VoteAttestation)
}

async function updateHeaderV2() {
    let [wallet] = await ethers.getSigners();
    const LightNode = await ethers.getContractFactory("LightNodeV2");
    let addr = await deployLightNodeV2(43100000)
    // let addr = "0x67A4D89f262566fB1a5ae7fD0EA303B32C1e0A7f"
    let lightNode = LightNode.attach(addr);
    const provider = new ethers.providers.JsonRpcProvider(uri);
    console.log(await lightNode.headerHeight());
    let block = await getBlock(43100200,provider);
    let VoteAttestation1 = {
        VoteAddressSet:253,
        uncompressPublicKeys:[
        "0x16f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c206e3243550cf23c92614b99a436622826a6105de683cc2f12db79179889fce81420d49569dd563577116c94d3ad3866d",
        "0x179974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd9985002300b5146732641e846d417e985324f07f5af44b64876804320437e806d0bd1ff8f9e6dbc13e03997fba459528e0dd83e86",
        "0x0a39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b1833ab10b8b992f09c9dedb4771313d43be03a5d806d26cd4960294a6b8eb47b6c2fb68110c77bcccbfb68068d1ec27d",
        "0x003af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf0852399b5639c6a7b16e4f7da83b9f194fb57709c49ba5905fd6b2a227581ddd42658a7a651da9c28c613be43bfc4c92",
        "0x19e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf05b7e2a0b7952769327c2823a9760e3d00f071d47b08706fb8ff3687a32640180be222f0e6597f554d36993ecb062b8c",
        "0x0d9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb164515a97928f994e665f4ef0371806a2d334b94029abe9ff13bda04983b6dc8d76c8c340b4058619c10a5a4813bd9f3",
        "0x0ade0f78a6b92b38c9f6d45ce8fb01da2b800100201cf0936b6b4b14c98af22edbe27df8aa197fca733891b5b6ca95db0dee753ad7bc28c28cc2e80fedd93b98f7e85a91c473410e9badb6b58140362eb69e0e75b23917c048b90e208e49964a",  
        ],
        Signature:"0x125de38374ae80ac93384d8c9131caaf6967739b5b033e8e155bb18c22bdd0e710d579fa99b2514d5bc8af50e4250083160f539dacbd18506abdc656ca48dd1726a7d1c8fbc4eca63512a2e1ac25a35cffc9b147d67df5d18b1e39339db254630cfd39b59b232c815ca62c91fed68c8edc7e80cd011d5f4906a85672045002a64cbe9191fd12ba39d84a3419a6d376270f6ced8dbebabc2a820af63c07dffefac6f04c5ccdc071831b5ba3580d0627068a113711e80ed3d9f68edeb55fafee54",
        Data:{
            SourceNumber:43100200,
            SourceHash:"0xe8ee73c240dfb7975a816a0685f99ae5d6987423f06587a776f5b3821cc7dc6a",
            TargetNumber:43100201,
            TargetHash:"0xd4fd34d37ac2e4521505ef7023d5cffb63c2d3cb4f475c6a0b5fd41f762291d7"
        }
    }
    let VoteAttestation2 = {
        VoteAddressSet:253,
        uncompressPublicKeys:[
            "0x16f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c206e3243550cf23c92614b99a436622826a6105de683cc2f12db79179889fce81420d49569dd563577116c94d3ad3866d",
            "0x179974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd9985002300b5146732641e846d417e985324f07f5af44b64876804320437e806d0bd1ff8f9e6dbc13e03997fba459528e0dd83e86",
            "0x0a39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b1833ab10b8b992f09c9dedb4771313d43be03a5d806d26cd4960294a6b8eb47b6c2fb68110c77bcccbfb68068d1ec27d",
            "0x003af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf0852399b5639c6a7b16e4f7da83b9f194fb57709c49ba5905fd6b2a227581ddd42658a7a651da9c28c613be43bfc4c92",
            "0x19e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf05b7e2a0b7952769327c2823a9760e3d00f071d47b08706fb8ff3687a32640180be222f0e6597f554d36993ecb062b8c",
            "0x0d9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb164515a97928f994e665f4ef0371806a2d334b94029abe9ff13bda04983b6dc8d76c8c340b4058619c10a5a4813bd9f3",
            "0x0ade0f78a6b92b38c9f6d45ce8fb01da2b800100201cf0936b6b4b14c98af22edbe27df8aa197fca733891b5b6ca95db0dee753ad7bc28c28cc2e80fedd93b98f7e85a91c473410e9badb6b58140362eb69e0e75b23917c048b90e208e49964a",  
            ],
        Signature:"0x068407677e45232778a1573724c5238e9ee8d2633049ae53c719c35afe26e39818c1c528522cfb43b94d7f25a579ceca02b43f96690108d0db809fe07927f0df5703b22f8346d103ffab8fbb66a2b052eba85fd76b3789bbdcfa26d6499036400b33ba01bae3252bc1f9bab2cc838839919b276bb789690b082f05732dc52b874b58325f5615d96128f694963ae8b2b608d87c0f17f682fe5a0f7d1b7a676687858adb1f25d7dcf284f4e6252aecdd06538de924c47a913d428b4eb6c2ad76a8",
        Data:{
            SourceNumber:43100201,
            SourceHash:"0xd4fd34d37ac2e4521505ef7023d5cffb63c2d3cb4f475c6a0b5fd41f762291d7",
            TargetNumber:43100202,
            TargetHash:"0x874086af5f4cb5efcc584dc890138d1255508ca16e99680dcdf36d11d527c8c7"
        }
    }
    let UpdateHeader = {
        headers: [block],
        voteAttestations:[VoteAttestation1,VoteAttestation2]
    }

    console.log(await lightNode.getBLSPublicKeysByNumber(43100201))
    await (await lightNode.updateBlockHeader(await lightNode.getHeadersBytes(UpdateHeader))).wait()

    console.log(await lightNode.headerHeight());


   let block1 = await getBlock(43100400,provider);
    let VoteAttestation3 = {
        VoteAddressSet:253,
        uncompressPublicKeys:[
        "0x16f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c206e3243550cf23c92614b99a436622826a6105de683cc2f12db79179889fce81420d49569dd563577116c94d3ad3866d",
        "0x179974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd9985002300b5146732641e846d417e985324f07f5af44b64876804320437e806d0bd1ff8f9e6dbc13e03997fba459528e0dd83e86",
        "0x0a39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b1833ab10b8b992f09c9dedb4771313d43be03a5d806d26cd4960294a6b8eb47b6c2fb68110c77bcccbfb68068d1ec27d",
        "0x003af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf0852399b5639c6a7b16e4f7da83b9f194fb57709c49ba5905fd6b2a227581ddd42658a7a651da9c28c613be43bfc4c92",
        "0x19e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf05b7e2a0b7952769327c2823a9760e3d00f071d47b08706fb8ff3687a32640180be222f0e6597f554d36993ecb062b8c",
        "0x0d9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb164515a97928f994e665f4ef0371806a2d334b94029abe9ff13bda04983b6dc8d76c8c340b4058619c10a5a4813bd9f3",
        "0x0ade0f78a6b92b38c9f6d45ce8fb01da2b800100201cf0936b6b4b14c98af22edbe27df8aa197fca733891b5b6ca95db0dee753ad7bc28c28cc2e80fedd93b98f7e85a91c473410e9badb6b58140362eb69e0e75b23917c048b90e208e49964a",  
        ],
        Signature:"0x12e09279b07c964c9314056b5a7a6abf9bdd0f5ce98cb37d4956858905b5c2ec5f3ffc1cf1ce21d7f7b1db551f4c37f10b17d08653345220072c835c9dd4beef2b8516e8a156268b329446bc5b5ea946aee31e8ba8bb07b0204980bf75708b8009de6e703f0953232a8fde3daf5f529b5be651077f32478a147352dfb4e9c3fc6c728407ede389782b6123b8d91f6626027de7465422f0ccd77f83e8c495f2720ffc403bcfc990f17d8d07e0fc9890bdfc9ab4dce464b9fcf72ed4586bc2a45a",
        Data:{
            SourceNumber:43100400,
            SourceHash:"0x8a8c09cabcf312726a07a391ea32863ea74c2b8f31a175137f539eb58dfbd5a3",
            TargetNumber:43100401,
            TargetHash:"0x43726916951bb5514bd3315f3ff0004e7e438e8c06acd6cb44e241b8428664b1"
        }
    }

    let VoteAttestation4 = {
        VoteAddressSet:253,
        uncompressPublicKeys:[
            "0x16f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c206e3243550cf23c92614b99a436622826a6105de683cc2f12db79179889fce81420d49569dd563577116c94d3ad3866d",
            "0x179974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd9985002300b5146732641e846d417e985324f07f5af44b64876804320437e806d0bd1ff8f9e6dbc13e03997fba459528e0dd83e86",
            "0x0a39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b1833ab10b8b992f09c9dedb4771313d43be03a5d806d26cd4960294a6b8eb47b6c2fb68110c77bcccbfb68068d1ec27d",
            "0x003af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf0852399b5639c6a7b16e4f7da83b9f194fb57709c49ba5905fd6b2a227581ddd42658a7a651da9c28c613be43bfc4c92",
            "0x19e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf05b7e2a0b7952769327c2823a9760e3d00f071d47b08706fb8ff3687a32640180be222f0e6597f554d36993ecb062b8c",
            "0x0d9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb164515a97928f994e665f4ef0371806a2d334b94029abe9ff13bda04983b6dc8d76c8c340b4058619c10a5a4813bd9f3",
            "0x0ade0f78a6b92b38c9f6d45ce8fb01da2b800100201cf0936b6b4b14c98af22edbe27df8aa197fca733891b5b6ca95db0dee753ad7bc28c28cc2e80fedd93b98f7e85a91c473410e9badb6b58140362eb69e0e75b23917c048b90e208e49964a",  
            ],
        Signature:"0x0efc456eb48869fb08a5072a2efafa03192f958e9414ecc474899fe79c3018c0283bc0d916dcaecdcd06a37dd5562a3703b61f2489aa0c905fbabe8dd0ed308e74682567d3d14b06d1bd04f8e5cca2711974bde26e6bfe706ab3eade9d11c2c00da628d0968021ba8169e9d6f47a131af3bb5e0aac4ee04c5ba795ee50a98ca1ad4b9e2c86cf126c5434d4ee1330e86e1804376804d5a39a9483978ebc7fb69d1441388c240ed10824005a642de2657e960cb5959a5c0ca7fb8ea35765ea8f85",
        Data:{
            SourceNumber:43100401,
            SourceHash:"0x43726916951bb5514bd3315f3ff0004e7e438e8c06acd6cb44e241b8428664b1",
            TargetNumber:43100402,
            TargetHash:"0xf3d28bbed97363261defdc642e962dd98e0ad2a4c39b9434c83007a3a56a006b"
        }
    }
    let UpdateHeader1 = {
        headers: [block1],
        voteAttestations:[VoteAttestation3,VoteAttestation4]
    }
    console.log(await lightNode.getBLSPublicKeysByNumber(43100402))
    await (await lightNode.updateBlockHeader(await lightNode.getHeadersBytes(UpdateHeader1))).wait()
    console.log(await lightNode.headerHeight());
}

async function  verifyV2() {
    let [wallet] = await ethers.getSigners();
    console.log("begin ...");
    const LightNode = await ethers.getContractFactory("LightNodeV2");
    let lightNode = LightNode.attach("0x0d7dc773b94869a80295400d630e618d998f51be");
    let tx = "0x5d0e31b0514103419d676c918abc011287768a5e611fd71cb72d3cbf8468fc0c";
    let txReceipt = await getTxReceipt(tx, uri);
    console.log("txReceipt ==", txReceipt)
    let proof = await getReceiptProof(tx, uri);
    console.log("proof ===", proof)
    const Encode = await ethers.getContractFactory("Encode");
    let encode = Encode.attach("0xc539511dd84455fe216486cb95Ad84418Ae69f5A");
    let txReceiptBytes = await encode.encodeReceipt(txReceipt);
    let ReceiptProof = {
            txReceipt: txReceiptBytes,
            receiptType: txReceipt.receiptType,
            keyIndex: proof.key,
            proof : proof.proof
        }
    console.log("ReceiptProof == ", ReceiptProof)
    const provider = new ethers.providers.JsonRpcProvider(uri);
    let block1 = await getBlock(43100400,provider);
    let VoteAttestation3 = {
        VoteAddressSet:253,
        uncompressPublicKeys:[
            "0x16f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c206e3243550cf23c92614b99a436622826a6105de683cc2f12db79179889fce81420d49569dd563577116c94d3ad3866d",
            "0x179974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd9985002300b5146732641e846d417e985324f07f5af44b64876804320437e806d0bd1ff8f9e6dbc13e03997fba459528e0dd83e86",
            "0x0a39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b1833ab10b8b992f09c9dedb4771313d43be03a5d806d26cd4960294a6b8eb47b6c2fb68110c77bcccbfb68068d1ec27d",
            "0x003af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf0852399b5639c6a7b16e4f7da83b9f194fb57709c49ba5905fd6b2a227581ddd42658a7a651da9c28c613be43bfc4c92",
            "0x19e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf05b7e2a0b7952769327c2823a9760e3d00f071d47b08706fb8ff3687a32640180be222f0e6597f554d36993ecb062b8c",
            "0x0d9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb164515a97928f994e665f4ef0371806a2d334b94029abe9ff13bda04983b6dc8d76c8c340b4058619c10a5a4813bd9f3",
            "0x0ade0f78a6b92b38c9f6d45ce8fb01da2b800100201cf0936b6b4b14c98af22edbe27df8aa197fca733891b5b6ca95db0dee753ad7bc28c28cc2e80fedd93b98f7e85a91c473410e9badb6b58140362eb69e0e75b23917c048b90e208e49964a",  
        ],
        Signature:"0x12e09279b07c964c9314056b5a7a6abf9bdd0f5ce98cb37d4956858905b5c2ec5f3ffc1cf1ce21d7f7b1db551f4c37f10b17d08653345220072c835c9dd4beef2b8516e8a156268b329446bc5b5ea946aee31e8ba8bb07b0204980bf75708b8009de6e703f0953232a8fde3daf5f529b5be651077f32478a147352dfb4e9c3fc6c728407ede389782b6123b8d91f6626027de7465422f0ccd77f83e8c495f2720ffc403bcfc990f17d8d07e0fc9890bdfc9ab4dce464b9fcf72ed4586bc2a45a",
        Data:{
            SourceNumber:43100400,
            SourceHash:"0x8a8c09cabcf312726a07a391ea32863ea74c2b8f31a175137f539eb58dfbd5a3",
            TargetNumber:43100401,
            TargetHash:"0x43726916951bb5514bd3315f3ff0004e7e438e8c06acd6cb44e241b8428664b1"
        }
    }
    let VoteAttestation4 = {
        VoteAddressSet:253,
        uncompressPublicKeys:[
                "0x16f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c206e3243550cf23c92614b99a436622826a6105de683cc2f12db79179889fce81420d49569dd563577116c94d3ad3866d",
                "0x179974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd9985002300b5146732641e846d417e985324f07f5af44b64876804320437e806d0bd1ff8f9e6dbc13e03997fba459528e0dd83e86",
                "0x0a39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b1833ab10b8b992f09c9dedb4771313d43be03a5d806d26cd4960294a6b8eb47b6c2fb68110c77bcccbfb68068d1ec27d",
                "0x003af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf0852399b5639c6a7b16e4f7da83b9f194fb57709c49ba5905fd6b2a227581ddd42658a7a651da9c28c613be43bfc4c92",
                "0x19e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf05b7e2a0b7952769327c2823a9760e3d00f071d47b08706fb8ff3687a32640180be222f0e6597f554d36993ecb062b8c",
                "0x0d9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb164515a97928f994e665f4ef0371806a2d334b94029abe9ff13bda04983b6dc8d76c8c340b4058619c10a5a4813bd9f3",
                "0x0ade0f78a6b92b38c9f6d45ce8fb01da2b800100201cf0936b6b4b14c98af22edbe27df8aa197fca733891b5b6ca95db0dee753ad7bc28c28cc2e80fedd93b98f7e85a91c473410e9badb6b58140362eb69e0e75b23917c048b90e208e49964a",  
            ],
        Signature:"0x0efc456eb48869fb08a5072a2efafa03192f958e9414ecc474899fe79c3018c0283bc0d916dcaecdcd06a37dd5562a3703b61f2489aa0c905fbabe8dd0ed308e74682567d3d14b06d1bd04f8e5cca2711974bde26e6bfe706ab3eade9d11c2c00da628d0968021ba8169e9d6f47a131af3bb5e0aac4ee04c5ba795ee50a98ca1ad4b9e2c86cf126c5434d4ee1330e86e1804376804d5a39a9483978ebc7fb69d1441388c240ed10824005a642de2657e960cb5959a5c0ca7fb8ea35765ea8f85",
        Data:{
            SourceNumber:43100401,
            SourceHash:"0x43726916951bb5514bd3315f3ff0004e7e438e8c06acd6cb44e241b8428664b1",
            TargetNumber:43100402,
            TargetHash:"0xf3d28bbed97363261defdc642e962dd98e0ad2a4c39b9434c83007a3a56a006b"
        }
    }
    let UpdateHeader1 = {
        headers: [block1],
        voteAttestations:[VoteAttestation3,VoteAttestation4]
    }

    let ProofData = {
            updateHeader:UpdateHeader1,
            receiptProof:ReceiptProof
    }
    console.log("ProofData == ", ProofData)
    console.log(await lightNode.verifyProofData(await lightNode.getBytes(ProofData)));
}

async function  deployLightNode(initBlock:number) {
    let [wallet] = await ethers.getSigners();
    // let mpt_addr = await deployMpt()
    let mpt_addr = "0x81D26E2387059CF43ADA1c11c12D5d6627184fA1";
    const LightNode = await ethers.getContractFactory("LightNode");
    const lightNode = await LightNode.deploy();
    await lightNode.connect(wallet).deployed();
    console.log("lightNode Implementation deployed on:", lightNode.address);
    const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");
    const provider = new ethers.providers.JsonRpcProvider(uri);
    let lastHeader = await getBlock(initBlock, provider);
    let second = await getBlock(initBlock - epochNum, provider);
    let initHeaders: Array<BlockHeader> = new Array<BlockHeader>();
    initHeaders.push(second);
    initHeaders.push(lastHeader);
    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        minEpochBlockExtraDataLen,
        wallet.address,
        mpt_addr,
        initHeaders,
    ]);
    const lightNodeProxy = await LightNodeProxy.deploy(lightNode.address, initData);
    await lightNodeProxy.connect(wallet).deployed();
    return lightNodeProxy.address;
}

async function  deployLightNodeV2(innitBlock:number) {
    let [wallet] = await ethers.getSigners();
    const LightNode = await ethers.getContractFactory("LightNodeV2");
    // let mpt_addr = await deployMpt()
    let mpt_addr = "0x81D26E2387059CF43ADA1c11c12D5d6627184fA1";
    const provider = new ethers.providers.JsonRpcProvider(uri);
    const impl = await LightNode.deploy();
    await impl.connect(wallet).deployed();
    console.log("impl",impl.address)
    const LightNodeProxy = await ethers.getContractFactory("LightNodeProxy");
    let lastHeader = await getBlock(innitBlock , provider);
    let second = await getBlock((innitBlock - epochNum), provider);
    let initHeaders: Array<BlockHeader> = new Array<BlockHeader>();
    initHeaders.push(second)
    initHeaders.push(lastHeader);
    let initData = LightNode.interface.encodeFunctionData("initialize", [
        chainId,
        wallet.address,
        mpt_addr,
        initHeaders,
    ]);
    const lightNodeProxy = await LightNodeProxy.deploy(impl.address, initData);
    await lightNodeProxy.connect(wallet).deployed();
    return lightNodeProxy.address;
}

async function  deployMpt() {
    let [wallet] = await ethers.getSigners();
    const MPTVerify = await ethers.getContractFactory("MPTVerify");
    const mPTVerify = await MPTVerify.deploy();
    await mPTVerify.connect(wallet).deployed();
    return mPTVerify.address;
}

async function getVoteDataRlpHash() {
    let [wallet] = await ethers.getSigners();
    console.log("begin ...");
    const LightNode = await ethers.getContractFactory("LightNodeV2");
    const lightNode = await LightNode.deploy();
    await lightNode.connect(wallet).deployed();
    let data = {
        SourceNumber:43100198,
        SourceHash:"0x9419d23fdef4ded6289e08d04df3bc1695a607b60685c4228ceb70edfb155d89",
        TargetNumber:43100199,
        TargetHash:"0x6c583197e8fe512d05a748501449669c2eb75a98cb49c7861a66caa6a257c22d"
    }
    let hash = await lightNode.getVoteDataRlpHash(data)
    console.log(hash);
}

async function getBlockHash() {
    let [wallet] = await ethers.getSigners();
    console.log("begin ...");
    const LightNode = await ethers.getContractFactory("LightNodeV2");
    const lightNode = await LightNode.deploy();
    await lightNode.connect(wallet).deployed();
    const provider = new ethers.providers.JsonRpcProvider(uri);
    let blockNumber = 43338288
    let blockHeader = await getBlock(blockNumber, provider);
    console.log("rpcHeader",blockHeader)
    let hash = await lightNode.getBlockHash(blockHeader)
    console.log(hash);
}

async function updateHeaderV1() {
    let [wallet] = await ethers.getSigners();
    // let addr = ""
    const LightNode = await ethers.getContractFactory("LightNode");
    let addr = await deployLightNode(43100000);
    let lightNode = LightNode.attach(addr);
    await updateHeader(wallet,lightNode);
}
async function updateHeader(wallet: SignerWithAddress, lightNode: Contract) {
    const provider = new ethers.providers.JsonRpcProvider(uri);
    let last: BigNumber = await lightNode.headerHeight();
    let headers: Array<BlockHeader> = new Array<BlockHeader>();

    for (let i = 0; i < 5; i++) {
        let lastHeader = await getBlock(last.toNumber() + epochNum + i, provider);
        headers.push(lastHeader);
    }
    await (await lightNode.updateBlockHeader(await lightNode.getHeadersBytes(headers))).wait();
    console.log(await lightNode.headerHeight());
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
