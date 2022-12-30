// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "./interface/ILightClientManager.sol";
import "./interface/ILightNode.sol";


contract LightClientManager is ILightClientManager, Initializable,UUPSUpgradeable {
    mapping(uint256 => address) public lightClientContract;

    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), " only owner");
        _;
    }

    function initialize() public initializer
    {
        _changeAdmin(msg.sender);
    }

    function register(uint256 _chainId, address _contract) external onlyOwner {
        lightClientContract[_chainId] = _contract;
    }

    function updateBlockHeader(uint256 _chainId, bytes memory _blockHeader) external override {
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        lightNode.updateBlockHeader(_blockHeader);
    }

    function updateLightClient(uint256 _chainId, bytes memory _data) external override {
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        lightNode.updateLightClient(_data);
    }

    function verifyProofData(uint256 _chainId, bytes memory _receiptProof) external view override
    returns (bool success, string memory message, bytes memory logs) {
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        return lightNode.verifyProofData(_receiptProof);
    }

    function clientState(uint256 _chainId) external view override returns(bytes memory) {
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);

        return lightNode.clientState();
    }

    function headerHeight(uint256 _chainId) external view override returns (uint256){
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);

        return lightNode.headerHeight();
    }

    function verifiableHeaderRange(uint256 _chainId) external view override returns (uint256, uint256){
        require(lightClientContract[_chainId] != address(0), "not register");
        ILightNode lightNode = ILightNode(lightClientContract[_chainId]);
        (uint256 min,uint256 max) = lightNode.verifiableHeaderRange();
        return(min,max);
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightClientManager: only Admin can upgrade");
    }

    function changeAdmin(address _admin) external onlyOwner checkAddress(_admin) {
        _changeAdmin(_admin);
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }

}