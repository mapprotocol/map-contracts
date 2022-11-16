// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Verify.sol";

// import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    uint256 public constant EPOCH_NUM = 64;

    address public mptVerify;

    uint256 public minValidBlocknum;

    uint256 public minEpochBlockExtraDataLen;

    mapping(uint256 => bytes) public validators;

    uint256 internal _lastSyncedBlock;

    struct ProofData {
        Verify.BlockHeader header;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    constructor(
        uint256 _minEpochBlockExtraDataLen,
        address _controller,
        address _mptVerify
    ) {
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
    }

    function initialize(
        uint256 _minEpochBlockExtraDataLen,
        address _controller,
        address _mptVerify,
        Verify.BlockHeader memory _header
    ) public initializer {
        require(minEpochBlockExtraDataLen == 0, "already initialized");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
        _initBlock(_header);
    }

    function togglePause(bool _flag) public onlyOwner returns (bool) {
        if (_flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function updateBlockHeader(bytes memory _blockHeadersBytes)
        external
        override
        whenNotPaused
    {
        Verify.BlockHeader memory _blockHeader = abi.decode(
            _blockHeadersBytes,
            (Verify.BlockHeader)
        );

        _lastSyncedBlock += EPOCH_NUM;

        require(
            _blockHeader.number == _lastSyncedBlock,
            "invalid syncing block"
        );

        require(_verifyBlockHeaders(_blockHeader), "blocks verify fail");

        validators[(_lastSyncedBlock + 1) / EPOCH_NUM] = Verify.getValidators(
            _blockHeader.extraData
        );

        emit UpdateBlockHeader(tx.origin, _blockHeader.number);
    }

    function verifyProofData(bytes memory _receiptProof)
        external
        view
        override
        returns (
            bool success,
            string memory message,
            bytes memory logs
        )
    {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));

        Verify.BlockHeader memory header = proof.header;

        require(
            header.number >= minValidBlocknum &&
                header.number <= maxCanVerifyNum(),
            "Can not verify blocks"
        );

        success = _verifyBlockHeaders(header);
        if (!success) {
            message = "invalid proof blocks";
        } else {
            bytes32 rootHash = bytes32(header.receiptsRoot);
            (success, logs) = Verify.validateProof(
                rootHash,
                proof.receiptProof,
                mptVerify
            );

            if (!success) {
                message = "mpt verify fail";
            }
        }
    }

    function _initBlock(Verify.BlockHeader memory _header) internal {
        require(_lastSyncedBlock == 0, "already init");
        require((_header.number + 1) % EPOCH_NUM == 0, "invalid init block");

        bytes memory validator = Verify.getValidators(_header.extraData);
        require(validator.length > 20, "no validator init");

        validators[(_header.number + 1) / EPOCH_NUM] = validator;

        _lastSyncedBlock = _header.number;

        minValidBlocknum = _header.number + 1;
    }

    function _verifyBlockHeaders(Verify.BlockHeader memory _blockHeader)
        internal
        view
        returns (bool)
    {
        require(
            Verify.validateHeader(_blockHeader, minEpochBlockExtraDataLen),
            "invalid bock header"
        );

        address signer = Verify.recoverSigner(_blockHeader);

        require(
            Verify.containValidator(
                validators[_lastSyncedBlock / EPOCH_NUM],
                signer
            ),
            "invalid block header singer"
        );

        return true;
    }

    function getBytes(ProofData memory _proof)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(_proof);
    }

    function getHeadersBytes(Verify.BlockHeader memory _blockHeader)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(_blockHeader);
    }

    function headerHeight() external view override returns (uint256) {
        return _lastSyncedBlock;
    }

    function maxCanVerifyNum() public view returns (uint256) {
        return _lastSyncedBlock + EPOCH_NUM;
    }

     function verifiableHeaderRange() external view override returns (uint256, uint256){
        return (minValidBlocknum,maxCanVerifyNum());
     }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }

    function changeAdmin(address _admin) public onlyOwner {
        require(_admin != address(0), "zero address");

        _changeAdmin(_admin);
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
