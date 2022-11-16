// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Verify.sol";

//import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    uint256 internal constant EPOCH_NUM = 200;

    address public mptVerify;

    uint256 public chainId;

    uint256 public minValidBlocknum;

    uint256 public minEpochBlockExtraDataLen;

    mapping(uint256 => bytes) public validators;

    uint256 internal _lastSyncedBlock;

    struct ProofData {
        Verify.BlockHeader[] headers;
        Verify.ReceiptProof receiptProof;
    }

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    constructor(
        uint256 _chainId,
        uint256 _minEpochBlockExtraDataLen,
        address _controller,
        address _mptVerify
    ) {
        chainId = _chainId;
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
    }

    function initialize(
        uint256 _chainId,
        uint256 _minEpochBlockExtraDataLen,
        address _controller,
        address _mptVerify,
        Verify.BlockHeader[2] memory _headers
    ) public initializer {
        require(chainId == 0, "already initialized");
        require(_controller != address(0), "_controller zero address");
        require(_mptVerify != address(0), "_mptVerify zero address");
        mptVerify = _mptVerify;
        _changeAdmin(_controller);
        chainId = _chainId;
        minEpochBlockExtraDataLen = _minEpochBlockExtraDataLen;

        _initBlock(_headers);
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
        Verify.BlockHeader[] memory _blockHeaders = abi.decode(
            _blockHeadersBytes,
            (Verify.BlockHeader[])
        );
        _lastSyncedBlock += EPOCH_NUM;

        require(
            _blockHeaders[0].number == _lastSyncedBlock,
            "invalid start block"
        );

        uint256 min = validators[_lastSyncedBlock - EPOCH_NUM].length / 40 + 1;

        require(_blockHeaders.length >= min, "not enough");

        require(_verifyBlockHeaders(_blockHeaders, min), "blocks verify fail");

        validators[_lastSyncedBlock] = Verify.getValidators(
            _blockHeaders[0].extraData
        );

        emit UpdateBlockHeader(tx.origin, _blockHeaders[0].number);
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
                (headers[headers.length - 1].number % EPOCH_NUM);
            if (validators[recently].length == 0) {
                min = validators[recently - EPOCH_NUM].length / 40 + 1;
            } else {
                min = validators[recently].length >
                    validators[recently - EPOCH_NUM].length
                    ? validators[recently - EPOCH_NUM].length / 40 + 1
                    : validators[recently].length / 40 + 1;
            }
        }
        require(headers.length >= min, "not enough");

        success = _verifyBlockHeaders(headers, min);

        if (!success) {
            message = "invalid proof blocks";
        } else {
            bytes32 rootHash = bytes32(headers[0].receiptsRoot);
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

    function _initBlock(Verify.BlockHeader[2] memory _headers) internal {
        require(_lastSyncedBlock == 0, "already init");

        require(_headers[0].number + EPOCH_NUM == _headers[1].number);

        for (uint256 i = 0; i < 2; i++) {
            require(_headers[i].number % EPOCH_NUM == 0);

            require(_headers[i].extraData.length > minEpochBlockExtraDataLen);

            validators[_headers[i].number] = Verify.getValidators(
                _headers[i].extraData
            );
        }

        minValidBlocknum = _headers[1].number;

        _lastSyncedBlock = _headers[1].number;
    }

    function _verifyBlockHeaders(
        Verify.BlockHeader[] memory _blockHeaders,
        uint256 _min
    ) internal view returns (bool) {
        address[] memory miners = new address[](_blockHeaders.length);

        uint256 start = _blockHeaders[0].number;

        uint256 preGasLimt = _blockHeaders[0].gasLimit;

        bytes32 preBlockHash;

        uint256 preBlockTime;

        bytes memory vals;

        for (uint256 i = 0; i < _min; i++) {
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
                Verify.validateHeader(
                    _blockHeaders[i],
                    preGasLimt,
                    minEpochBlockExtraDataLen
                ),
                "invalid block"
            );
            preGasLimt = _blockHeaders[i].gasLimit;

            uint256 recently = _blockHeaders[i].number -
                (_blockHeaders[i].number % EPOCH_NUM);

            if (
                _blockHeaders[i].number % EPOCH_NUM >
                validators[recently - EPOCH_NUM].length / 40
            ) {
                vals = validators[recently];
            } else {
                vals = validators[recently - EPOCH_NUM];
            }

            require(
                Verify.containValidator(
                    vals,
                    _blockHeaders[i].miner,
                    _blockHeaders[i].number % (vals.length / 20)
                ),
                "invalid miner"
            );
            require(
                Verify.verifyHeaderSignature(_blockHeaders[i], chainId),
                "invalid Signature"
            );

            if (i > 0) {
                require(
                    !_isRepeat(miners, _blockHeaders[i].miner, i),
                    "miner repeat"
                );
            }

            miners[i] = _blockHeaders[i].miner;
        }

        return true;
    }

    function _isRepeat(
        address[] memory _miners,
        address _miner,
        uint256 _limit
    ) private pure returns (bool) {
        for (uint256 i = 0; i < _limit; i++) {
            if (_miners[i] == _miner) {
                return true;
            }
        }

        return false;
    }

    function getBytes(ProofData memory _proof)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(_proof);
    }

    function getHeadersBytes(Verify.BlockHeader[] memory _blockHeaders)
        public
        pure
        returns (bytes memory)
    {
        return abi.encode(_blockHeaders);
    }

    function headerHeight() external view override returns (uint256) {
        return _lastSyncedBlock;
    }

    function maxCanVerifyNum() public view returns (uint256) {
        return
            _lastSyncedBlock +
            EPOCH_NUM +
            (validators[_lastSyncedBlock].length / 40);
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
