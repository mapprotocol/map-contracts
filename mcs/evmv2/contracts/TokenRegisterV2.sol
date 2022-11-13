// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "./interface/ITokenRegisterV2.sol";
import "./interface/IVaultTokenV2.sol";
import "./utils/Utils.sol";

contract TokenRegisterV2 is Ownable, ITokenRegisterV2 {
    using SafeMath for uint;

    uint256 constant MAX_RATE_UNI = 1000000;

    struct FeeRate {
        uint256     lowest;
        uint256     highest;
        uint256     rate;      // unit is parts per million
    }

    struct Token {
        bool        mintable;
        uint8       decimals;
        address     vaultToken;

        mapping(uint256 => FeeRate) fees;
        // chain_id => decimals
        mapping(uint256 => uint8) tokenDecimals;
        // chain_id => token
        mapping(uint256 => bytes) mappingTokens;
    }

    uint public immutable selfChainId = block.chainid;

    //Source chain to Relay chain address
    // [chain_id => [source_token => map_token]]
    mapping(uint256 => mapping(bytes => address)) public tokenMappingList;

    mapping(address => Token) public tokenList;

    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }

    function registerToken(address _token, address _vaultToken, bool _mintable)
    external
    onlyOwner checkAddress(_token) checkAddress(_vaultToken) {
        Token storage token = tokenList[_token];
        address tokenAddress = IVaultTokenV2(_vaultToken).getTokenAddress();
        require(_token == tokenAddress, "invalid vault token");

        token.vaultToken = _vaultToken;
        token.decimals = IERC20Metadata(_token).decimals();
        token.mintable = _mintable;
    }

    function mapToken(address _token, uint256 _fromChain, bytes memory _fromToken, uint8 _decimals)
    external
    onlyOwner {
        require(!Utils.checkBytes(_fromToken, bytes("")), "invalid from token");
        Token storage token = tokenList[_token];
        require(token.vaultToken != address(0), "invalid map token");
        token.tokenDecimals[_fromChain] = _decimals;
        token.mappingTokens[_fromChain] = _fromToken;
        tokenMappingList[_fromChain][_fromToken] = _token;
    }

    function setTokenFee( address _token, uint256 _toChain, uint _lowest, uint _highest,uint _rate) external onlyOwner {
        Token storage token = tokenList[_token];
        require(token.vaultToken != address(0), "invalid map token");
        require(_highest >= _lowest, 'invalid highest and lowest');
        require(_rate <= MAX_RATE_UNI, 'invalid proportion value');

        token.fees[_toChain] = FeeRate(_lowest, _highest, _rate);
    }


    function getToChainToken(address _token, uint256 _toChain)
    external override
    view
    returns (bytes memory _toChainToken){
        _toChainToken = tokenList[_token].mappingTokens[_toChain];
    }

    function getToChainAmount(address _token, uint256 _amount, uint256 _toChain)
    external override
    view
    returns (uint256){
        uint256 decimalsFrom = tokenList[_token].decimals;
        uint256 decimalsTo = tokenList[_token].tokenDecimals[_toChain];
        if (decimalsFrom == decimalsTo) {
            return _amount;
        }
        return _amount.mul(10 ** decimalsTo).div(10 ** decimalsFrom);
    }

    function getRelayChainToken(uint256 _fromChain, bytes memory _fromToken)
    external override
    view
    returns (address token){
        token = tokenMappingList[_fromChain][_fromToken];
    }

    function getRelayChainAmount(address _token, uint256 _fromChain, uint256 _amount)
    external override view returns (uint256){
        uint256 decimalsFrom = tokenList[_token].tokenDecimals[_fromChain];
        uint256 decimalsTo = tokenList[_token].decimals;
        if (decimalsFrom == decimalsTo) {
            return _amount;
        }
        return _amount.mul(10 ** decimalsTo).div(10 ** decimalsFrom);
    }

    function checkMintable(address _token)
    external override view returns (bool) {
        return tokenList[_token].mintable;
    }

    function getVaultToken(address _token)
    external override view returns (address) {
        return tokenList[_token].vaultToken;
    }

    function getTokenFee(address _token, uint256 _amount, uint256 _toChain)
    external view override returns (uint256){
        FeeRate memory feeRate = tokenList[_token].fees[_toChain];

        uint256 fee = _amount.mul(feeRate.rate).div(MAX_RATE_UNI);
        if (fee > feeRate.highest){
            return feeRate.highest;
        }else if (fee < feeRate.lowest){
            return feeRate.lowest;
        }
        return fee;
    }




}