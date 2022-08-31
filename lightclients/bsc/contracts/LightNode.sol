// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Verify.sol";
//import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    uint256 internal constant epochNum = 200;

    bytes[2] public validators;

    mapping(uint256 => bytes32) receiptsRoots;

    bytes32 internal preBlockHash;

    uint256 internal preGasLimit;

    uint256 internal preTime;

    uint256 internal chainId;

    uint256 internal lastSyncedBlock;

    uint256 public minEpochBlockExtraDataLen;

    struct ProofData {
        uint256 blockNum;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    event UpdateBlockHeader(uint256 blockHeight);

    constructor(uint256 _chainId, uint256 _minEpochBlockExtraDataLen) {
        chainId = _chainId;
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
    }

    function initialize(uint256 _chainId, uint256 _minEpochBlockExtraDataLen)
        public
        initializer
    {
        require(chainId == 0, "already initialized");
        _changeAdmin(msg.sender);
        chainId = _chainId;
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
    }

    function initBlock(
        bytes memory preValidators,
        Verify.BlockHeader memory _blockHeader
    ) public onlyOwner {
        require(lastSyncedBlock == 0, "already init");

        require(_blockHeader.number % epochNum == 0, "invalid init block");

        validators[0] = preValidators;

        preGasLimit = _blockHeader.gasLimit;

        preBlockHash = Verify.getBlockHash(_blockHeader);

        validators[1] = Verify.getValidators(_blockHeader.extraData);

        lastSyncedBlock = _blockHeader.number;

        preTime = _blockHeader.timestamp;
    }

    function trigglePause(bool flag) public onlyOwner returns (bool) {
        if (flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
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

        bytes32 rootHash = receiptsRoots[proof.blockNum];

        if (uint256(rootHash) == 0) {
            success = false;

            message = "unconfirm block";
        } else {
            (success, logs) = Verify.validateProof(rootHash, proof.receiptProof);

            if (!success) {
                message = "mpt verify fail";
            }
        }
    }

    function updateBlockHeader(Verify.BlockHeader[] memory _blockHeaders)
        external
        override
        whenNotPaused
    {
        uint256 min = validators[0].length > validators[1].length
            ? validators[1].length / 40 + 1
            : validators[0].length / 40 + 1;

        require(_blockHeaders.length >= min, "not enough");

        lastSyncedBlock++;

        uint256 _lastSyncedBlock = lastSyncedBlock;

        uint256 _preGasLimit = preGasLimit;

        bytes32 _preBlockHash = preBlockHash;

        uint256 _pretime = preTime;

        address[] memory miners = new address[](_blockHeaders.length);

        for (uint256 i = 0; i < _blockHeaders.length; i++) {
            require(
                _blockHeaders[i].number == _lastSyncedBlock + i,
                "invalid bolck number"
            );

            require(
                _blockHeaders[i].timestamp > _pretime,
                "invalid block time"
            );

            require(
                _blockHeaders[i].parentHash.length == 32 &&
                    bytes32(_blockHeaders[i].parentHash) == _preBlockHash,
                "invalid parentHash"
            );

            require(
                Verify.validHeader(
                    _blockHeaders[i],
                    _preGasLimit,
                    minEpochBlockExtraDataLen
                ),
                "invalid block"
            );

            bytes memory _validators;

            if (
                _blockHeaders[i].number % epochNum > validators[0].length / 40
            ) {
                _validators = validators[1];
            } else {
                _validators = validators[0];
            }

            require(
                Verify.containValidator(
                    _validators,
                    _blockHeaders[i].miner,
                    _blockHeaders[i].number % (_validators.length / 20)
                ),
                "invalid miner"
            );

            _preBlockHash = Verify.getBlockHash(_blockHeaders[i]);

            require(
                Verify.verifyHeaderSignature(_blockHeaders[i], chainId),
                "invalid Signature"
            );

            _preGasLimit = _blockHeaders[i].gasLimit;

            _pretime = _blockHeaders[i].timestamp;

            if (i == 0) {
                preBlockHash = _preBlockHash;
                preGasLimit = _preGasLimit;
                preTime = _pretime;

                if (_blockHeaders[i].number % epochNum == 0) {
                    validators[0] = validators[1];
                    validators[1] = Verify.getValidators(
                        _blockHeaders[i].extraData
                    );
                }
                receiptsRoots[_blockHeaders[i].number] = bytes32(
                    _blockHeaders[i].receiptsRoot
                );
            } else {
                require(
                    !isRepeat(miners, _blockHeaders[i].miner, i),
                    "miner repeat"
                );
            }

            miners[i] = _blockHeaders[i].miner;
        }

        emit UpdateBlockHeader(lastSyncedBlock);
    }

    function getBytes(ProofData memory proof)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(proof);
    }


    function isRepeat(
        address[] memory _miners,
        address miner,
        uint256 limit
    ) private pure returns (bool) {
        for (uint256 i = 0; i < limit; i++) {
            if (_miners[i] == miner) {
                return true;
            }
        }

        return false;
    }

    function headerHeight() external view override returns (uint256) {
        return lastSyncedBlock;
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
