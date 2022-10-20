

let initData =
    {   "epoch":1,
        "epoch_size":1000,
        "threshold":3,
        "validators":
            [
                {"weight":1,"address":"0x053af2b1ccbacba47c659b977e93571c89c49654","g1_pub_key":{"x":"0x25480e726faeaecdba3d09bd8079c17153a99914400ee7c68d6754d29d7832c1","y":"0x2b9804718e2cb3f65221781647a8c3455cf3090519b15a34ef43b1dde7e3c287"}},
                {"weight":1,"address":"0xb47adf1e504601ff7682b68ba7990410b92cd958","g1_pub_key":{"x":"0x120bf5a2d293b4d444448304d5d04775bfff199676180111112ec0db7f8a6a69","y":"0x2685ac2dc25dc5dd06a6b4777d542d4f4afdf92847b9b7c98f5ecaf4d908f6d7"}},
                {"weight":1,"address":"0xf655fc7c95c70a118f98b46ca5028746284349a5","g1_pub_key":{"x":"0x03dda4ec969ff7950903131caf2cc0df1d91c569be382cab67df539e94a45835","y":"0x156b522a45ed4a625a7b5906d64046dce1c112a1dddb72972ecb670145a16042"}},
                {"weight":1,"address":"0xb243f68e8e3245464d21b79c7ceae347ecc08ea6","g1_pub_key":{"x":"0x28681fcac6825e2a6711b2ef0d3a22eae527c41ecccdeb4e69dfff4002219d8b","y":"0x131f98eaf9323bf171e947401f0e6b1951f4c8f8aa525b677f1c811c88358e37"}}
            ]
    }

let lightNodeProxyAddress = "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9";

module.exports = {
    initData,
    lightNodeProxyAddress
}