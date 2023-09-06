// SPDX-License-Identifier: MIT
pragma solidity 0.8.7;

interface IChildToken {
    function deposits(bytes32 deposit)external returns(uint256);
}
contract Relay {

  address public childToken;
  mapping(bytes32 => bool) public deposits;

  struct  MapSwapOut {
        uint256  fromChain;
        uint256  toChain;
        bytes32 orderId;
        bytes token;
        bytes from;
        bytes to;
        uint256 amount;
        bytes swapData ;
  }

  event mapSwapOut(
        uint256 indexed fromChain, // from chain
        uint256 indexed toChain, // to chain
        bytes32 orderId, // order id
        bytes token, // token to transfer
        bytes from, // source chain from address
        bytes to,
        uint256 amount,
        bytes swapData // swap data, used on target chain dex.
  ); 

  constructor(address _childToken){
        childToken = _childToken;
  }


  function relay(bytes calldata depositData) external {
        bytes32 hash = keccak256(depositData);
        require(!deposits[hash],"already relay");

        deposits[hash] = true;

        require(IChildToken(childToken).deposits(hash) > 0,"not deposit");

        bytes memory eventBytes = depositData[32:];
        MapSwapOut memory m;
        (m.fromChain,m.toChain,m.orderId,m.token,m.from,m.to,m.amount,m.swapData) =
        abi.decode(eventBytes,(uint256,uint256,bytes32,bytes,bytes,bytes,uint256,bytes));

        emit mapSwapOut(m.fromChain,m.toChain,m.orderId,m.token,m.from,m.to,m.amount,m.swapData);

  }
  


}