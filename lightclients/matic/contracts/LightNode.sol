// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Verify.sol";

// import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    uint256 internal constant EPOCH_NUM = 64;

    address public mptVerify;

    uint256 public minValidBlocknum;

    uint256 public minEpochBlockExtraDataLen;

    mapping(uint256 => bytes) public validators;

    uint256 internal lastSyncedBlock;

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
        Verify.BlockHeader memory header
    ) public initializer {
        require(minEpochBlockExtraDataLen == 0, "already initialized");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
        initBlock(header);
    }

    function togglePause(bool flag) public onlyOwner returns (bool) {
        if (flag) {
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

        lastSyncedBlock += EPOCH_NUM;

        require(
            _blockHeader.number == lastSyncedBlock,
            "invalid syncing block"
        );

        require(verifyBlockHeaders(_blockHeader), "blocks verify fail");

        validators[(lastSyncedBlock + 1) / EPOCH_NUM] = Verify.getValidators(
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

        success = verifyBlockHeaders(header);
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

    function initBlock(Verify.BlockHeader memory header) internal {
        require(lastSyncedBlock == 0, "already init");
        require((header.number + 1) % EPOCH_NUM == 0, "invalid init block");

        bytes memory validator = Verify.getValidators(header.extraData);
        require(validator.length > 20, "no validator init");

        validators[(header.number + 1) / EPOCH_NUM] = validator;

        lastSyncedBlock = header.number;

        minValidBlocknum = header.number + 1;
    }

    function verifyBlockHeaders(Verify.BlockHeader memory _blockHeader)
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
                validators[lastSyncedBlock / EPOCH_NUM],
                signer
            ),
            "invalid block header singer"
        );

        return true;
    }

    function getBytes(ProofData memory proof)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(proof);
    }

    function getHeadersBytes(Verify.BlockHeader memory _blockHeader)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(_blockHeader);
    }

    function headerHeight() external view override returns (uint256) {
        return lastSyncedBlock;
    }

    function maxCanVerifyNum() public view returns (uint256) {
        return lastSyncedBlock + EPOCH_NUM;
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
