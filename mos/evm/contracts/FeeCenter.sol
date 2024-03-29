// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./interface/IFeeCenter.sol";
import "./utils/TransferHelper.sol";



contract FeeCenter is IFeeCenter, AccessControl, Initializable,Ownable {
    using SafeMath for uint;
    mapping(uint => mapping (address => gasFee)) chainTokenGasFee;
    //token to vtoken
    mapping(address => address) tokenVault;

    //id : 0 VToken  1:relayer
    mapping(uint => Rate) distributeRate;

    event SetChainTokenGasFee(uint256 to ,address token,uint256 lowest,uint256 highest,uint256 proportion);
    event SetTokenVault(address token,address valut);
    event SetDistributeRate(uint256 id ,address to,uint256 rate);

    function setChainTokenGasFee(uint to, address token, uint lowest, uint highest,uint proportion) external onlyOwner {
        require(highest >= lowest, 'Invalid highest and lowest');
        require(proportion <= 1000000, 'Invalid proportion value');
        chainTokenGasFee[to][token] = gasFee(lowest,highest,proportion);
        emit SetChainTokenGasFee(to,token,lowest,highest,proportion);
    }

    function setTokenVault(address token,address tVault) external onlyOwner{
        tokenVault[token] = tVault;
        emit SetTokenVault(token,tVault);
    }

    function _getTokenFee(uint _to, address _token, uint _amount) internal view returns (uint){
        gasFee memory gf =  chainTokenGasFee[_to][_token];
        uint fee = _amount.mul(gf.proportion).div(1000000);
        if (fee > gf.highest){
            return gf.highest;
        }else if (fee < gf.lowest){
            return gf.lowest;
        }
        return fee;
    }

    function getTokenFee(uint to, address token, uint amount) external view override returns (uint){
        uint fee = _getTokenFee(to,token,amount);
        require(fee <= amount, "amount too small");
        return fee;
    }

    function getVaultToken(address token) external view override returns(address vault){
        return tokenVault[token];
    }

    function doDistribute(address token,uint amount) external override onlyOwner{
        address vaultAddress = tokenVault[token];
        require(vaultAddress != address(0), "vault not set");

        Rate memory vaultRate = distributeRate[0];
        uint vaultAmount = amount.mul(vaultRate.rate).div(1000000);
        TransferHelper.safeTransfer(token,vaultAddress,vaultAmount);

        Rate memory relayerRate = distributeRate[1];
        uint relayerAmount = amount.mul(relayerRate.rate).div(1000000);
        TransferHelper.safeTransfer(token,relayerRate.feeAddress,relayerAmount);
    }

    function getDistribute(uint id, address token) external view override returns(address feeAddress, uint rates){
        Rate memory rate = distributeRate[id];
        if (id == 0) {
            address vaultAddress = tokenVault[token];
            rate.feeAddress = vaultAddress;
        }
        return(rate.feeAddress, rate.rate);
    }

    function setDistributeRate(uint id, address to, uint rate) external onlyOwner {
        distributeRate[id] = Rate(to, rate);
        require(distributeRate[0].rate.add(distributeRate[1].rate)<= 1000000, 'Invalid rate value');
        emit SetDistributeRate(id,to,rate);
    }

}