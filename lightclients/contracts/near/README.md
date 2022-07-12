updateBlockHeader(bytes memory _blackHeader);

_blackHeader 为rpc api  next_light_client_block 获取的结果 borsh 编码 和 rainbow-bridge 一样

执行updateBlockHeader 先 执行 initWithValidators ，再执行 initWithBlock 和 rainbow-bridge 一样

verifyProofData(bytes memory _receiptProof) 这个方法我改成了view 类型

 rainbow-bridge 传的是 bytes memory proofData, uint64 blockHeight；

//

http post http://127.0.0.1:3030/ jsonrpc=2.0 method=EXPERIMENTAL_light_client_proof params:="{"type": "receipt", "receipt_id": <receipt_id>, "receiver_id": <receiver_id>, "light_client_head": < light_client_head>}" id="dontcare"

获取 proofData

blockHeight 是链上提交了区块头的区块高度  应该也是 light_client_head 的区块高度

_receiptProof = abi.encode(blockHeight,proofData)



跑 npx hardhat run ./test/onChainTest.js --network map_test 连接map测试网测试
