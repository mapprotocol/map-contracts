// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/LedgerInfoLib.sol";
import "./lib/Types.sol";
import "./lib/RLPReader.sol";
import "./LedgerInfo.sol";
import "./Provable.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    using RLPReader for RLPReader.RLPItem;

    LedgerInfo private _ledgerInfo;
    Provable private _mptVerify;

    State private _state;
    LedgerInfoLib.Committee private _committee;

    // pow block number => pow block hash
    mapping(uint256 => bytes32) public finalizedBlocks;

    event UpdateBlockHeader(address indexed account, uint256 indexed blockHeight);

    function initialize(
        address controller,
        address ledgerInfoUtil,
        address mptVerify,
        LedgerInfoLib.EpochState memory committee,
        LedgerInfoLib.LedgerInfoWithSignatures memory ledgerInfo
    ) external override initializer {
        require(controller != address(0), "invalid controller address");
        require(ledgerInfoUtil != address(0), "invalid ledgerInfoUtil address");
        require(mptVerify != address(0), "invalid mptVerify address");

        _changeAdmin(controller);
        _ledgerInfo = LedgerInfo(ledgerInfoUtil);
        _mptVerify = Provable(mptVerify);

        require(committee.epoch > 0 && committee.epoch == ledgerInfo.epoch, "invalid committee epoch");
        require(ledgerInfo.pivot.height > 0, "block number too small");

        // init client state
        _state.epoch = ledgerInfo.epoch;
        _state.round = ledgerInfo.round;
        _state.earliestBlockNumber = ledgerInfo.pivot.height;
        _state.finalizedBlockNumber = ledgerInfo.pivot.height;
        _state.blocks = 1;
        _state.maxBlocks = 3 * 1440 * 30; // about 1 month
        finalizedBlocks[ledgerInfo.pivot.height] = ledgerInfo.pivot.blockHash;

        // init committee
        LedgerInfoLib.updateCommittee(_committee, committee);
    }

    modifier onlyInitialized() {
        require(_getInitializedVersion() > 0, "uninitialized");
        _;
    }

    function setMaxBlocks(uint256 val) external onlyOwner {
        _state.maxBlocks = val;
    }

    function updateLightClient(bytes memory _data) external override {
        LedgerInfoLib.LedgerInfoWithSignatures memory ledgerInfo = abi.decode(_data, (LedgerInfoLib.LedgerInfoWithSignatures));
        relayPOS(ledgerInfo);
    }

    function relayPOS(LedgerInfoLib.LedgerInfoWithSignatures memory ledgerInfo) public override onlyInitialized whenNotPaused {
        require(ledgerInfo.epoch == _state.epoch, "epoch mismatch");
        require(ledgerInfo.round > _state.round, "round mismatch");

        bytes memory message = _ledgerInfo.bcsEncode(ledgerInfo);
        (bytes memory signature, bytes[] memory publicKeys) = LedgerInfoLib.packSignatures(_committee, ledgerInfo);
        bool verified = _ledgerInfo.aggregateVerifyBLS(signature, message, publicKeys);
        require(verified, "invalid BLS signatures");

        if (ledgerInfo.nextEpochState.epoch == 0) {
            _state.round = ledgerInfo.round;
        } else {
            require(ledgerInfo.nextEpochState.epoch == _state.epoch + 1, "invalid epoch for the next committee");
            _state.epoch = ledgerInfo.nextEpochState.epoch;
            _state.round = 0; // indicate to relay pos block in next epoch
            LedgerInfoLib.updateCommittee(_committee, ledgerInfo.nextEpochState);
        }

        // in case that pow block may not generate for a long time
        if (ledgerInfo.pivot.height > _state.finalizedBlockNumber) {
            _state.finalizedBlockNumber = ledgerInfo.pivot.height;
            _state.blocks++;
            finalizedBlocks[ledgerInfo.pivot.height] = ledgerInfo.pivot.blockHash;
        }

        removeBlockHeader(1);
        emit UpdateBlockHeader(tx.origin,ledgerInfo.pivot.height);
    }

    function updateBlockHeader(bytes memory _blockHeader) external override {
        bytes[] memory headers = abi.decode(_blockHeader, (bytes[]));
        relayPOW(headers);
    }

    function relayPOW(bytes[] memory headers) public override onlyInitialized whenNotPaused {
        Types.BlockHeaderWrapper memory head = _validateHeaders(headers);

        if (finalizedBlocks[head.height] == bytes32(0)) {
            _state.blocks++;
            finalizedBlocks[head.height] = keccak256(headers[0]);
        }

        removeBlockHeader(1);

        emit UpdateBlockHeader(tx.origin,head.height);
    }

    function _validateHeaders(bytes[] memory headers) private view returns (Types.BlockHeaderWrapper memory head) {
        require(headers.length > 0, "empty block headers");

        Types.BlockHeaderWrapper memory tail = Types.rlpDecodeBlockHeader(headers[headers.length - 1]);
        uint256 expectedBlockNumber = tail.height;
        bytes32 expectedBlockHash = finalizedBlocks[tail.height];
        require(expectedBlockHash != bytes32(0), "tail block not found");
        for (uint256 i = 0; i < headers.length; i++) {
            // validate in reverse order
            uint256 index = headers.length - 1 - i;
            Types.BlockHeaderWrapper memory header = Types.rlpDecodeBlockHeader(headers[index]);

            require(header.height == expectedBlockNumber, "block number mismatch");
            require(keccak256(headers[index]) == expectedBlockHash, "block hash mismatch");

            expectedBlockNumber--;
            expectedBlockHash = header.parentHash;
        }

        head = Types.rlpDecodeBlockHeader(headers[0]);
        require(head.height > _state.earliestBlockNumber, "block number too small");
    }

    function removeBlockHeader(uint256 limit) public override {
        require(limit > 0, "limit is zero");

        if (_state.blocks <= _state.maxBlocks) {
            return;
        }

        uint256 numRemoved = _state.blocks - _state.maxBlocks;
        if (numRemoved > limit) {
            numRemoved = limit;
        }

        uint256 earliest = _state.earliestBlockNumber;
        for (; numRemoved > 0; earliest++) {
            if (finalizedBlocks[earliest] != 0) {
                delete finalizedBlocks[earliest];
                numRemoved--;
            }
        }

        _state.blocks -= numRemoved;

        while (finalizedBlocks[earliest] == 0) {
            earliest++;
        }

        _state.earliestBlockNumber = earliest;
    }

    function verifyReceiptProof(Types.ReceiptProof memory proof) public view override returns (bool) {
        Types.BlockHeaderWrapper memory head = _validateHeaders(proof.headers);

        return _mptVerify.proveReceipt(
            head.deferredReceiptsRoot,
            proof.blockIndex,
            proof.blockProof,
            proof.receiptsRoot,
            proof.index,
            proof.receipt,
            proof.receiptProof
        );
    }

    function verifyProofData(bytes memory receiptProof) external view override returns (
        bool success, string memory message, bytes memory rlpLogs
    ) {
        Types.ReceiptProof memory proof = abi.decode(receiptProof, (Types.ReceiptProof));
        success = verifyReceiptProof(proof);

        if (success) {
            rlpLogs = RLPReader.toRlpItem(proof.receipt).toList()[4].toRlpBytes();
        } else {
            message = "failed to verify mpt";
        }
    }

    function clientState() external view override returns(bytes memory) {
        return abi.encode(
            _state.epoch,
            _state.round,
            _state.earliestBlockNumber,
            _state.finalizedBlockNumber,
            _state.blocks,
            _state.maxBlocks
        );
    }

    function state() external view override returns(State memory) {
        return _state;
    }

    function headerHeight() external view override returns (uint256 height) {
        return _state.finalizedBlockNumber;
    }

    function verifiableHeaderRange() external view override returns (uint256, uint256) {
        return (_state.earliestBlockNumber, _state.finalizedBlockNumber);
    }

    function nearestPivot(uint256 height) public view override returns (uint256) {
        require(height >= _state.earliestBlockNumber, "block already pruned");
        require(height <= _state.finalizedBlockNumber, "block not finalized yet");

        while (finalizedBlocks[height] == 0) {
            height++;
        }

        return height;
    }

    function finalizedState(bytes memory data) external view override returns (bytes memory) {
        uint256 height = abi.decode(data, (uint256));
        uint256 pivot = nearestPivot(height);
        return abi.encode(pivot);
    }

    /** common code copied from other light nodes ********************/
    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    function togglePause(bool flag) external onlyOwner returns (bool) {
        if (flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
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
