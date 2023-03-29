// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Verify.sol";


contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {

    uint256 internal constant MAX_SAVED_EPOCH_NUM = 18083; // three month

    address public mptVerify;

    uint256 public minValidBlocknum;

    mapping(uint256 => Verify.Validator[]) public validators;

    uint256 internal _lastSyncedBlock;

    uint256 public chainId;

    address private _pendingAdmin;

    event ChangePendingAdmin(
        address indexed previousPending,
        address indexed newPending
    );
    event AdminTransferred(address indexed previous, address indexed newAdmin);

    struct ProofData {
        Verify.QuorumCert quorumCert;
        Verify.BlockHeader header;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    constructor() {}

    function initialize(
        uint256 _chainId,
        address _controller,
        address _mptVerify,
        Verify.BlockHeader calldata _header,
        Verify.Validator[] calldata _validators
    ) external initializer {
        require(_chainId > 0, "invalid _chainId");
        require(chainId == 0, "already initialized");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        chainId = _chainId;
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
        _initBlock(_header, _validators);
    }

    function togglePause(bool _flag) external onlyOwner returns (bool) {
        if (_flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function updateBlockHeader(
        bytes memory _blockHeadersBytes
    ) external override whenNotPaused {
        (
            Verify.BlockHeader memory _blockHeader,
            Verify.QuorumCert memory _quorumCert,
            Verify.Validator[] memory _validators
        ) = abi.decode(
                _blockHeadersBytes,
                (Verify.BlockHeader, Verify.QuorumCert, Verify.Validator[])
            );

        require(
            _lastSyncedBlock + Verify.EPOCH_NUM == _blockHeader.number,
            "invalid blockHeader number"
        );
        // verify blockHeader
        require(Verify._validateHeader(_blockHeader), "invalid blockHeader");
        // verify epoch_validators
        require(
            Verify._verifyValidators(_validators, _blockHeader.extraData),
            "invalid validators"
        );

        //verify quorumCert
        bytes32 blockHash = Verify._getBlockHash(_blockHeader,_blockHeader.extraData);
        require(
            Verify._verifyQuorumCert(
                blockHash,
                _quorumCert,
                validators[(_blockHeader.number - 1) / Verify.EPOCH_NUM]
            ),
            "invalid QuorumCert"
        );
        //set validators
        _storeValidators(_blockHeader.number, _validators);

        _lastSyncedBlock = _blockHeader.number;

        _removeExcessEpochValidators();

        emit UpdateBlockHeader(tx.origin, _blockHeader.number);
    }

    function verifyProofData(
        bytes memory _receiptProof
    )
        external
        view
        override
        returns (bool success, string memory message, bytes memory logs)
    {
        ProofData memory proof = abi.decode(_receiptProof, (ProofData));

        Verify.BlockHeader memory header = proof.header;

        if (
            header.number < minValidBlocknum ||
            header.number > maxCanVerifyNum()
        ) {
            success = false;
            message = "out of verify range";
            return (success, message, logs);
        }

        success = Verify._validateHeader(header);

        if (!success) {
            message = "invalid blockHeader";
            return (success, message, logs);
        }
        //verify quorumCert
        bytes32 blockHash = Verify._getBlockHash(header,header.extraData);

        if (
            !Verify._verifyQuorumCert(
                blockHash,
                proof.quorumCert,
                validators[(header.number - 1) / Verify.EPOCH_NUM]
            )
        ) {
            success = false;
            message = "invalid QuorumCert";
            return (success, message, logs);
        }
        // verify mpt
        if (success) {
            bytes32 rootHash = bytes32(header.receiptsRoot);
            (success, logs) = Verify._validateProof(
                rootHash,
                proof.receiptProof,
                mptVerify
            );

            if (!success) {
                message = "mpt verification failed";
            }
        }
    }

    function _initBlock(
        Verify.BlockHeader memory _header,
        Verify.Validator[] memory _validators
    ) internal {
        // verify blockHeader
        require(Verify._validateHeader(_header), "invalid blockHeader");
        // verify epoch_validators
        require(
            Verify._verifyValidators(_validators, _header.extraData),
            "invalid validators"
        );

        _storeValidators(_header.number, _validators);

        _lastSyncedBlock = _header.number;

        minValidBlocknum = _header.number + 1;
    }

    function _storeValidators(
        uint256 _blockNumber,
        Verify.Validator[] memory _validators
    ) internal {
        Verify.Validator[] storage v = validators[_blockNumber / Verify.EPOCH_NUM];

        for (uint256 i = 0; i < _validators.length; i++) {
            v.push(_validators[i]);
        }
    }

    function _removeExcessEpochValidators() internal {
        uint256 remove = _lastSyncedBlock / Verify.EPOCH_NUM - MAX_SAVED_EPOCH_NUM;

        if (validators[remove].length > 0) {
            minValidBlocknum += Verify.EPOCH_NUM;
            delete validators[remove];
        }
    }

    function getBytes(
        ProofData calldata _proof
    ) external pure returns (bytes memory) {
        return abi.encode(_proof);
    }

    function decode(bytes calldata p) public pure returns(ProofData memory _proof){
        return abi.decode(p,(ProofData));
    }

    function getHeadersBytes(
        Verify.BlockHeader memory _blockHeader,
        Verify.QuorumCert memory _quorumCert,
        Verify.Validator[] memory _validators
    ) external pure returns (bytes memory) {
        return abi.encode(_blockHeader, _quorumCert, _validators);
    }

    function headerHeight() external view override returns (uint256) {
        return _lastSyncedBlock;
    }

    function maxCanVerifyNum() public view returns (uint256) {
        return _lastSyncedBlock + Verify.EPOCH_NUM;
    }

    function verifiableHeaderRange()
        external
        view
        override
        returns (uint256, uint256)
    {
        return (minValidBlocknum, maxCanVerifyNum());
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }

    function changeAdmin() external {
        require(_pendingAdmin == msg.sender, "only pendingAdmin");
        emit AdminTransferred(_getAdmin(), _pendingAdmin);
        _changeAdmin(_pendingAdmin);
    }

    function pendingAdmin() external view returns (address) {
        return _pendingAdmin;
    }

    function setPendingAdmin(address pendingAdmin_) external onlyOwner {
        require(
            pendingAdmin_ != address(0),
            "Ownable: pendingAdmin is the zero address"
        );
        emit ChangePendingAdmin(_pendingAdmin, pendingAdmin_);
        _pendingAdmin = pendingAdmin_;
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
