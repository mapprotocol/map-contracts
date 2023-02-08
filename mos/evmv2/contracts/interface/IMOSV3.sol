// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

interface IMOSV3 {
    struct CallData {
        bytes target;
        bytes callData;
        uint256 gasLimit;
        uint256 value;
    }

    function transferOutToken(address _token, bytes memory _to, uint _amount, uint _toChain) external;
    function transferOutNative(bytes memory _to, uint _toChain) external payable;
    function depositToken(address _token, address to, uint _amount) external;
    function depositNative(address _to) external payable ;
    function transferOut(uint256 _toChain,CallData memory _callData) external payable  returns(bool);


    event mapTransferOut(uint256 indexed fromChain, uint256 indexed toChain, bytes32 orderId,
        bytes token, bytes from, bytes to, uint256 amount, bytes toChainToken);

    event mapTransferIn(uint256 indexed fromChain, uint256 indexed toChain, bytes32 orderId,
        address token, bytes from,  address to, uint256 amount);

    event mapDepositOut(uint256 indexed fromChain, uint256 indexed toChain, bytes32 orderId,
        address token, bytes from, address to, uint256 amount);

    event mapDataOut(uint256 indexed fromChain, uint256 indexed toChain,bytes32 orderId, bytes callData);

    event mapExecuteIn(uint256 indexed fromChain, uint256 indexed toChain, bytes32 orderId, bool executeTag);

}