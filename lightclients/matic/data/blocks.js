let { BigNumber } = require("ethers");


let initBlock = {
  parentHash: '0x63f50501bfe6c055c424e26c31e560ae08f9ffcf438793d4a3294eaebc71a247',
  sha3Uncles: '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347',
  miner: '0x0000000000000000000000000000000000000000',
  stateRoot: '0x4769684a0189a80d24ece9ebcdbfaf83f92a0865f457fa6898fc91f50435fdf3',
  transactionsRoot: '0x5d3c3dda75fb4a073d3d0c1f1de32084acca2e5aacb8943cc9e02916f9439050',
  receiptsRoot: '0xa54c110770c0bd283262d30293335973309cdd8adcfeddc8793238a59b888e26',
  logsBloom: '0x01a8010020a05070042a2cda8422103f8000060018100a850183033014c91821c108930d9200c0818000c81900d463450020e2008812a404004007460932a422010901881183100800510919232920a310232020010403121531041080a57800005808200220849312004a0202885824008080cd4802ca2580810c10208811d20459b1408c8d00880c9a49413c200150089304880205080c306800630200102032000408100000268298100250203609060080e11007480710060402041826480213a21a0d1082531049841128e8d019f0e430404d0ba41cc232ab1e80006020eb110190012d1801802980008920280992040981342190854031087682144208',
  difficulty: BigNumber.from(23),
  number: BigNumber.from(34765823),
  gasLimit: BigNumber.from(26297771),
  gasUsed: BigNumber.from(7472921),
  timestamp: BigNumber.from(1666667011),
  extraData: '0xd682021183626f7288676f312e31382e31856c696e757800000000000000000000856730088a5c3191bd26eb482e45229555ce5700000000000000000000000000000000000000020208652a93baf5f1962849efcf5795eac7439a5e000000000000000000000000000000000000000102f70172f7f490653665c9bfac0666147c8af1f50000000000000000000000000000000000000001127685d6dd6683085da4b6a041efcef1681e5c9c00000000000000000000000000000000000000051efecb61a2f80aa34d3b9218b564a64d05946290000000000000000000000000000000000000000426c80cc193b27d73d2c40943acec77f4da2c5bd8000000000000000000000000000000000000000340314efbc35bc0db441969bce451bf0167efded1000000000000000000000000000000000000000146a3a41bd932244dd08186e4c19f1a7e48cbcdf4000000000000000000000000000000000000000160e274b09f701107a4b3226fcc1376ebda3cdd92000000000000000000000000000000000000000267b94473d81d0cd00849d563c94d0432ac988b49000000000000000000000000000000000000000372f93a2740e00112d5f2cef404c0aa16fae21fa40000000000000000000000000000000000000001742d13f0b2a19c823bdd362b16305e4704b97a380000000000000000000000000000000000000001794e44d1334a56fea7f4df12633b88820d0c588800000000000000000000000000000000000000037c7379531b2aee82e4ca06d4175d13b9cbeafd490000000000000000000000000000000000000004959a4d857b7071c38878beb9dc77051b5fed1dfd00000000000000000000000000000000000000019ead03f7136fc6b4bdb0780b00a1c14ae5a8b6d00000000000000000000000000000000000000005b9ede6f94d192073d8eaf85f8db677133d4832490000000000000000000000000000000000000002bc6044f4a1688d8b8596a9f7d4659e09985eebe60000000000000000000000000000000000000001bdbd4347b082d9d6bdf2da4555a37ce52a2e21200000000000000000000000000000000000000002c6869257205e20c2a43cb31345db534aecb49f6e0000000000000000000000000000000000000001e7e2cb8c81c10ff191a73fe266788c9ce62ec7540000000000000000000000000000000000000001ef46d5fe753c988606e6f703260d816af53b03eb0000000000000000000000000000000000000001f0245f6251bef9447a08766b9da2b07b28ad80b000000000000000000000000000000000000000047f6a735a11bd7ca0c06e36a9c369b1ce1c3bc02c39611bdd6bbbd429fabcd15a05ddcbb07d55aedd00173270c3830a9736e7fac94022ad50d453aabc059b6e5c01',
  mixHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
  nonce: '0x0000000000000000',
  baseFeePerGas: BigNumber.from('62648971877')
}



let addBlock = {
  parentHash: '0x6a2fc0f63cfe769be4c8ca954c26036685e829f3d2a43a335192edfe64feeb05',
  sha3Uncles: '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347',
  miner: '0x0000000000000000000000000000000000000000',
  stateRoot: '0x8d8706268589e28b257dfd0173946dae2a8a06911d83f3c1e03687876d32f4d5',
  transactionsRoot: '0x47914525802cd40bb2a2ba971bbb437746465b88ed1d2be67734c8c79be045d8',
  receiptsRoot: '0x30227c201e13edb81451acaa25079943148d341592ff726123fe5567f36649d1',
  logsBloom: '0x0e2021a21800112614680008a00848194100001811108024c8080c1910881b0a404cd9901c32c66c02272e104325102101018150000370020290212003b4a7d62c0440245bc764a9843020093239a1e6943524e0034401941a09a412041822821574cb0482040803284052032024984eb122106149128421958444b870e9c00a41110048da1d6c118cd92401bcc4237c009394840448002920598a490052007123428c1f748001c0220c81c099ac200adb446b1028230a441c0901b8feb07044fa04c71b940e5f2b21219a91683816046ee4d344cd4abd135850ba0a80b432003a12808320883000cc6990208126828296086305024a4b970430ba0d83f0d838',
  difficulty: BigNumber.from(23),
  number: BigNumber.from(34765887),
  gasLimit: BigNumber.from(27992878),
  gasUsed: BigNumber.from(8834326),
  timestamp: BigNumber.from(1666667143),
  extraData: '0xd682021083626f7288676f312e31372e36856c696e757800000000000000000000856730088a5c3191bd26eb482e45229555ce5700000000000000000000000000000000000000020208652a93baf5f1962849efcf5795eac7439a5e000000000000000000000000000000000000000102f70172f7f490653665c9bfac0666147c8af1f50000000000000000000000000000000000000001127685d6dd6683085da4b6a041efcef1681e5c9c00000000000000000000000000000000000000051efecb61a2f80aa34d3b9218b564a64d05946290000000000000000000000000000000000000000426c80cc193b27d73d2c40943acec77f4da2c5bd8000000000000000000000000000000000000000340314efbc35bc0db441969bce451bf0167efded1000000000000000000000000000000000000000146a3a41bd932244dd08186e4c19f1a7e48cbcdf4000000000000000000000000000000000000000160e274b09f701107a4b3226fcc1376ebda3cdd92000000000000000000000000000000000000000267b94473d81d0cd00849d563c94d0432ac988b49000000000000000000000000000000000000000372f93a2740e00112d5f2cef404c0aa16fae21fa40000000000000000000000000000000000000001742d13f0b2a19c823bdd362b16305e4704b97a380000000000000000000000000000000000000001794e44d1334a56fea7f4df12633b88820d0c588800000000000000000000000000000000000000037c7379531b2aee82e4ca06d4175d13b9cbeafd490000000000000000000000000000000000000004959a4d857b7071c38878beb9dc77051b5fed1dfd00000000000000000000000000000000000000019ead03f7136fc6b4bdb0780b00a1c14ae5a8b6d00000000000000000000000000000000000000005b9ede6f94d192073d8eaf85f8db677133d4832490000000000000000000000000000000000000002bc6044f4a1688d8b8596a9f7d4659e09985eebe60000000000000000000000000000000000000001bdbd4347b082d9d6bdf2da4555a37ce52a2e21200000000000000000000000000000000000000002c6869257205e20c2a43cb31345db534aecb49f6e0000000000000000000000000000000000000001e7e2cb8c81c10ff191a73fe266788c9ce62ec7540000000000000000000000000000000000000001ef46d5fe753c988606e6f703260d816af53b03eb0000000000000000000000000000000000000001f0245f6251bef9447a08766b9da2b07b28ad80b0000000000000000000000000000000000000000404bab48dc9e2cd35c2bc8a4fd67f18102effe865119ca160f25d5632f63723a749a2a754eaf4cdd8ab175506090ae7aaeed78871614f763b84081e2351b12b4801',
  mixHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
  nonce: '0x0000000000000000',
  baseFeePerGas: BigNumber.from('86040387264')
}


let addBlock1 = {
  parentHash: '0x79d969011c25eeef8b59626b288dab9f70decf057b61f84cec238af15a388662',
  sha3Uncles: '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347',
  miner: '0x0000000000000000000000000000000000000000',
  stateRoot: '0x71c7d853a6e42ac3189b56b964c6bbd5fb91788dfd17657bfef8f9212f0eff70',
  transactionsRoot: '0xc1e58f6b2b4f8e80dcdb872b7f276d50707834c735dc8051ceca8c8fad575dab',
  receiptsRoot: '0x7cfd1bd93d78affa74d196706999fafc31e1b0eb56a3cab0c35d1d2a6c0eabb9',
  logsBloom: '0x93ac23d285029208147830e0990481734d1239c6c2212009521a4000ea484ac460425b2244239034a1bd1531011099d41141faa00c8214842513cc0702ae01ce838e80282290d4085288528d5c2f72f428508c2851649e03b0dbec10aca3761841f8a00062b5086240826e0803a50949a22721614b001da5a101d03410a5c8172b0b0bb0c981b30ca0216800100a190428c18de38062804a10012865e0e05824a7611b0070a0646c875f450042bc00824147110c842458188274e1a2755a2c5169fc820e149a002248430e14428a540d8342af030c35f339c1b1c682c26463b0649901c04904000a207f010204048039a48649210501cc54087201115010282c',
  difficulty: BigNumber.from(23),
  number: BigNumber.from(34765951),
  gasLimit: BigNumber.from(29797255),
  gasUsed: BigNumber.from(26617469),
  timestamp: BigNumber.from(1666667275),
  extraData: '0xd682021083626f7288676f312e31392e32856c696e757800000000000000000000856730088a5c3191bd26eb482e45229555ce5700000000000000000000000000000000000000020208652a93baf5f1962849efcf5795eac7439a5e000000000000000000000000000000000000000102f70172f7f490653665c9bfac0666147c8af1f50000000000000000000000000000000000000001127685d6dd6683085da4b6a041efcef1681e5c9c00000000000000000000000000000000000000051efecb61a2f80aa34d3b9218b564a64d05946290000000000000000000000000000000000000000426c80cc193b27d73d2c40943acec77f4da2c5bd8000000000000000000000000000000000000000340314efbc35bc0db441969bce451bf0167efded1000000000000000000000000000000000000000146a3a41bd932244dd08186e4c19f1a7e48cbcdf4000000000000000000000000000000000000000160e274b09f701107a4b3226fcc1376ebda3cdd92000000000000000000000000000000000000000267b94473d81d0cd00849d563c94d0432ac988b49000000000000000000000000000000000000000372f93a2740e00112d5f2cef404c0aa16fae21fa40000000000000000000000000000000000000001742d13f0b2a19c823bdd362b16305e4704b97a380000000000000000000000000000000000000001794e44d1334a56fea7f4df12633b88820d0c588800000000000000000000000000000000000000037c7379531b2aee82e4ca06d4175d13b9cbeafd490000000000000000000000000000000000000004959a4d857b7071c38878beb9dc77051b5fed1dfd00000000000000000000000000000000000000019ead03f7136fc6b4bdb0780b00a1c14ae5a8b6d00000000000000000000000000000000000000005b9ede6f94d192073d8eaf85f8db677133d4832490000000000000000000000000000000000000002bc6044f4a1688d8b8596a9f7d4659e09985eebe60000000000000000000000000000000000000001bdbd4347b082d9d6bdf2da4555a37ce52a2e21200000000000000000000000000000000000000002c6869257205e20c2a43cb31345db534aecb49f6e0000000000000000000000000000000000000001e7e2cb8c81c10ff191a73fe266788c9ce62ec7540000000000000000000000000000000000000001ef46d5fe753c988606e6f703260d816af53b03eb0000000000000000000000000000000000000001f0245f6251bef9447a08766b9da2b07b28ad80b000000000000000000000000000000000000000048646d4168e34d3f43bf1c1d4bd0f881b59693bae08d08625d687dfab2557016279e6ddf13aca8d6f268bbb5a5f54809826f6b21a9171cd4bd3f9c584ebca2e3600',
  mixHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
  nonce: '0x0000000000000000',
  baseFeePerGas: BigNumber.from('48274340412')
}

let proofHeader = {
  parentHash: '0x6c55604b4c503d8286672edbc69512d5d68a727b80f67f8daf332b4b0cfdd4aa',
  sha3Uncles: '0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347',
  miner: '0x0000000000000000000000000000000000000000',
  stateRoot: '0xa0ac36d3a5af56e8b096ae1cfaf4e213bee9d1551664699167a0db8a4bceee47',
  transactionsRoot: '0x837dad3cfae36f23efa6974e7a72dd85b7636bf83ee35fb0505f1aba7347b445',
  receiptsRoot: '0x8ceeee3bcc463ec5ed40cf04a2114bfd6203bbcc63bcd65ec153cf425f99c632',
  logsBloom: '0x09aa08e24320988e70cdc24280312680502835bf09096311972b1cd811781f80600cb91da3468322700048b2008a21810f9ce4e4693266083410272c45662c55f82656e91942418b1853ccfbaaba19e0a46800624720d00942139781cb11683159513a21233de0369ea108c08d63ee2d21806003a0cfc720b089d9f84a9831b6853f1999a49e15a51d4d0ec43814576849254f9902006e496be0724025900082b3743910be83a96880a660b62b083ab0499860e8384d0a03a80149be678d08d280d1d79beec40303910db8850a08c850e44a48404cdadd32163bc8058226a24339921cb5994c210c127be8c02710b461b0a8b283069c80c29288280500bd6822',
  difficulty: BigNumber.from(23),
  number: BigNumber.from(34765840),
  gasLimit: BigNumber.from(26737757),
  gasUsed: BigNumber.from(12772607),
  timestamp: BigNumber.from(1666667049),
  extraData: '0xd682021083626f7288676f312e31372e36856c696e757800000000000000000063e5180a34237a6f6a9531a918b9730fb1b882d8b8682e141e6f3a885dd7a3b400a51ed7e1198912cfea82381d3a31c3693ae551bb313b7dcf9c5c2880491a4c00',   
  mixHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
  nonce: '0x0000000000000000',
  baseFeePerGas: BigNumber.from('83177857581')
}
//txHash 0xbf684bda3767bd3b756e03f441c1b36b68c09ef5795702af642eddf884053e29
let txReceipt = {
  receiptType: BigNumber.from(2),
  postStateOrStatus: '0x01',
  cumulativeGasUsed: BigNumber.from(12547264),
  bloom: '0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000001200000000000000000000000000000800000000000000000000100000000000000000000000000000000000000000000000000000000000080000000000000000000010000000000000000000000000000000000000000000000200000000000200000000000000000000000000000000000000000000000000000000000004000000100000000000001000000000000000000000000000000100000000000002000000000000000000000000000000000000000000000000000000000100000',
  logs: [
    {
      addr: '0x0000000000000000000000000000000000001010',
      topics: [
        '0x4dfe1bbbcf077ddc3e01291eea2d5c70c2b422b415d95645b9adcfd678cb1d63',
        '0x0000000000000000000000000000000000000000000000000000000000001010',
        '0x000000000000000000000000595bdb1dcdf935949d89c0c807f963b297fcd807',
        '0x0000000000000000000000001efecb61a2f80aa34d3b9218b564a64d05946290'
      ],
      data: '0x00000000000000000000000000000000000000000000000000014290a6c8c1900000000000000000000000000000000000000000000000000411f37827a9254d000000000000000000000000000000000000000000001a11dd9eb8fc5de66c250000000000000000000000000000000000000000000000000410b0e780e063bd000000000000000000000000000000000000000000001a11dd9ffb8d04af2db5'
    }
  ]
}

let proof = {
  proof: [
    '0xf871a0debfd3a70158311582c4b8bbc41f6e12f6c3b33cde9cea02e03bfb4f9a4611d6a06b24f0beaaec32a741ee9b2f9611c9c6ed8d5d741fc098b4af20803820cc863d808080808080a031d5a6e2c7c90ca79d2b9a967a3a1bed79eaa2db622371d8a6f472e5d491b0a08080808080808080',
    '0xf901d1a0aff414f73250372d5f61ce71c62b056c171380185ef4e8de168a4029ea2d0785a06aa875b1513ea94a172bccd64f210d2c75ce297207870181535f5e75bddb1ce6a07b5ecbeb600883dc196120d538edec3b93b8aa4491a6fe6da3f8ac5c32bb3881a070b2e76777aa8f98dabe62ee97b28b53874ae97c0eef8d4344e8209c919cfd0aa0031caa4b099e81b01dd08eb1e133dab3550b41d8d7564918b63acc4930f2dfb5a02c40061aae7d022c003a6270704db6049b1478b18c7bcbdd958f8ce5cbc2636ea00defebd64e7f387db278ec50056ba5a99979184c9e76e179e25d916bb4091046a0d004de3c22d2d1ea4d7c88f206d41aea535d2a7174710e57c18ef4e4ba186a40a0ef220808070e2fbb4fd40ab2b7c78c5570bb21e3b6cfc268b97bef04d8e429aca04aa4a4f47f7e862f2493d2c95c6633b6b0a1920029c627643d472ccfa2bc7ab5a0f4f254b162c7ac2746086f8cb8fd28daa4a2e085f7f0f4a8967af99f8a7c502ba04ab8384906bf1d4b6d2453c6cab9b699875c56db63cdb892cbbffb7236c6bb0da0779075bfe2b4a137fcc7976cb1d650cd0d9aeea8fd4c88849f0947e56c43205aa0e08b6b2705ee3bd97354883491dc4360315bee57a15a1435f8696509a3af5faf808080',
    '0xf9025320b9024f02f9024b0183bf74c0b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000001200000000000000000000000000000800000000000000000000100000000000000000000000000000000000000000000000000000000000080000000000000000000010000000000000000000000000000000000000000000000200000000000200000000000000000000000000000000000000000000000000000000000004000000100000000000001000000000000000000000000000000100000000000002000000000000000000000000000000000000000000000000000000000100000f90140f9013d940000000000000000000000000000000000001010f884a04dfe1bbbcf077ddc3e01291eea2d5c70c2b422b415d95645b9adcfd678cb1d63a00000000000000000000000000000000000000000000000000000000000001010a0000000000000000000000000595bdb1dcdf935949d89c0c807f963b297fcd807a00000000000000000000000001efecb61a2f80aa34d3b9218b564a64d05946290b8a000000000000000000000000000000000000000000000000000014290a6c8c1900000000000000000000000000000000000000000000000000411f37827a9254d000000000000000000000000000000000000000000001a11dd9eb8fc5de66c250000000000000000000000000000000000000000000000000410b0e780e063bd000000000000000000000000000000000000000000001a11dd9ffb8d04af2db5'
  ],
  key: '0x1c'
}





module.exports = {
  initBlock,
  txReceipt,
  proof,
  proofHeader,
  addBlock,
  addBlock1
};

