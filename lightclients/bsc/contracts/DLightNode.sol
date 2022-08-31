// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Verify.sol";
//import "hardhat/console.sol";

contract DLightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    uint256 internal constant epochNum = 200;

    mapping(uint256 => bytes) public validators;

    uint256 internal chainId;

    uint256 internal lastSyncedBlock;

    uint256 public minEpochBlockExtraDataLen;

    uint256 public minValidBlocknum;


    struct ProofData {
        Verify.BlockHeader[] headers;
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

    function initBlock(Verify.BlockHeader[2] memory headers) public onlyOwner {
        require(lastSyncedBlock == 0, "already init");

        require(headers[0].number + epochNum == headers[1].number);

        for (uint256 i = 0; i < 2; i++) {
            require(headers[i].number % epochNum == 0);

            require(headers[i].extraData.length > minEpochBlockExtraDataLen);

            validators[headers[i].number] = Verify.getValidators(
                headers[i].extraData
            );
        }

        minValidBlocknum = headers[1].number;

        lastSyncedBlock = headers[1].number;
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

        Verify.BlockHeader[] memory headers = proof.headers;

        require(
            headers[0].number >= minValidBlocknum &&
                headers[headers.length - 1].number <= maxCanVerifyNum(),
            "Can not verify blocks"
        );

        uint256 min;
        {
            uint256 recently = headers[headers.length - 1].number -
                (headers[headers.length - 1].number % epochNum);
            if (validators[recently].length == 0) {
                min = validators[recently - epochNum].length / 40 + 1;
            } else {
                min = validators[recently].length >
                    validators[recently - epochNum].length
                    ? validators[recently - epochNum].length / 40 + 1
                    : validators[recently].length / 40 + 1;
            }
        }
        require(headers.length >= min, "not enough");

        success = verifyBlockHeaders(headers);

        if (!success) {
            message = "invalid proof blocks";
        } else {
            bytes32 rootHash = bytes32(headers[0].receiptsRoot);
            (success, logs) = Verify.validateProof(
                rootHash,
                proof.receiptProof
            );

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
        lastSyncedBlock += epochNum;

        require(
            _blockHeaders[0].number == lastSyncedBlock,
            "invalid start block"
        );

        uint256 min = validators[lastSyncedBlock - epochNum].length / 40 + 1;

        require(_blockHeaders.length >= min, "not enough");

        require(verifyBlockHeaders(_blockHeaders), "blocks verify fail");

        validators[lastSyncedBlock] = Verify.getValidators(
            _blockHeaders[0].extraData
        );

        emit UpdateBlockHeader(_blockHeaders[0].number);
    }

    function verifyBlockHeaders(Verify.BlockHeader[] memory _blockHeaders)
        internal
        view
        returns (bool)
    {
        address[] memory miners = new address[](_blockHeaders.length);

        uint256 start = _blockHeaders[0].number;

        uint256 preGasLimt = _blockHeaders[0].gasLimit;

        bytes32 preBlockHash;

        uint256 preBlockTime;

        bytes memory _validators;

        for (uint256 i = 0; i < _blockHeaders.length; i++) {
            require(
                _blockHeaders[i].number == start + i,
                "invalid bolck number"
            );

            if (i > 0) {
                require(
                    _blockHeaders[i].timestamp > preBlockTime,
                    "invalid block time"
                );

                require(
                    _blockHeaders[i].parentHash.length == 32 &&
                        bytes32(_blockHeaders[i].parentHash) == preBlockHash,
                    "invalid parentHash"
                );
            }

            preBlockHash = Verify.getBlockHash(_blockHeaders[i]);

            preBlockTime = _blockHeaders[i].timestamp;
            require(
                Verify.validHeader(
                    _blockHeaders[i],
                    preGasLimt,
                    minEpochBlockExtraDataLen
                ),
                "invalid block"
            );
            preGasLimt = _blockHeaders[i].gasLimit;

            uint256 recently = _blockHeaders[i].number -
                (_blockHeaders[i].number % epochNum);

            if (
                _blockHeaders[i].number % epochNum >
                validators[recently - epochNum].length / 40
            ) {
                _validators = validators[recently];
            } else {
                _validators = validators[recently - epochNum];
            }

            require(
                Verify.containValidator(
                    _validators,
                    _blockHeaders[i].miner,
                    _blockHeaders[i].number % (_validators.length / 20)
                ),
                "invalid miner"
            );
            require(
                Verify.verifyHeaderSignature(_blockHeaders[i], chainId),
                "invalid Signature"
            );

            if (i > 0) {
                require(
                    !isRepeat(miners, _blockHeaders[i].miner, i),
                    "miner repeat"
                );
            }

            miners[i] = _blockHeaders[i].miner;
        }

        return true;
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

    function maxCanVerifyNum() public view returns (uint256) {
        return
            lastSyncedBlock +
            epochNum +
            (validators[lastSyncedBlock - epochNum].length / 40);
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
