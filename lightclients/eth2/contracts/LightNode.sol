// SPDX-License-Identifier: MIT

pragma solidity 0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";
import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";
import "./lib/Helper.sol";
import "./lib/Types.sol";

//import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    uint256 public constant FINALITY_PROOF_SIZE = 6;
    uint256 public constant EXECUTION_PROOF_SIZE = 4;
    uint256 public constant NEXT_SYNC_COMMITTEE_PROOF_SIZE = 5;
    uint256 public constant BLS_PUBKEY_LENGTH = 48;
    uint256 public constant MAX_BLOCK_SAVED = 32 * 256 * 30;
    uint256 public constant MAX_DELETE_COUNT = 100;

    address public mptVerify;
    uint64 public chainId;
    uint256 public finalizedExeHeaderNumber;

    // execution layer headers to update
    uint256 public exeHeaderStartNumber;
    uint256 public exeHeaderEndNumber;
    bytes32 public exeHeaderEndHash;

    Types.BeaconBlockHeader public finalizedBeaconHeader;
    Types.SyncCommittee[2] public syncCommittees;
    uint64 public initStage;
    bool public initialized;
    mapping(uint256 => bytes32) public finalizedExeHeaders;

    uint256 private startExeHeaderNumber;
    uint64 private curSyncCommitteeIndex;
    bytes32[] private syncCommitteePubkeyHashes;
    bool verifyUpdate;

    event UpdateLightClient(address indexed account, uint256 slot, uint256 height);
    event UpdateBlockHeader(address indexed account, uint256 start, uint256 end);

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    function initialize(
        uint64 _chainId,
        address _controller,
        address _mptVerify,
        Types.BeaconBlockHeader memory _finalizedBeaconHeader,
        uint256 _finalizedExeHeaderNumber,
        bytes32 _finalizedExeHeaderHash,
        bytes memory _curSyncCommitteeAggPubKey,
        bytes memory _nextSyncCommitteeAggPubKey,
        bytes32[] memory _syncCommitteePubkeyHashes, // divide 512 pubkeys into 3 parts: 171 + 171 + 170
        bool _verifyUpdate
    ) public initializer {
        require(_controller != address(0), "invalid controller");
        require(_mptVerify != address(0), "invalid mptVerify");
        require(_syncCommitteePubkeyHashes.length == 6, "invalid syncCommitteePubkeyHashes");
        for (uint256 i = 0; i < _syncCommitteePubkeyHashes.length; i++) {
            require(_syncCommitteePubkeyHashes[i] != bytes32(0), "invalid syncCommitteePubkeyHashes");
        }
        require(_curSyncCommitteeAggPubKey.length == BLS_PUBKEY_LENGTH, "invalid curSyncCommitteeAggPubKey");
        require(_nextSyncCommitteeAggPubKey.length == BLS_PUBKEY_LENGTH, "invalid nextSyncCommitteeAggPubKey");

        chainId = _chainId;
        mptVerify = _mptVerify;
        finalizedBeaconHeader = _finalizedBeaconHeader;
        finalizedExeHeaderNumber = _finalizedExeHeaderNumber;
        finalizedExeHeaders[_finalizedExeHeaderNumber] = _finalizedExeHeaderHash;
        startExeHeaderNumber = _finalizedExeHeaderNumber;

        syncCommittees[0].aggregatePubkey = _curSyncCommitteeAggPubKey;
        syncCommittees[1].aggregatePubkey = _nextSyncCommitteeAggPubKey;
        syncCommitteePubkeyHashes = _syncCommitteePubkeyHashes;
        curSyncCommitteeIndex = 0;

        verifyUpdate = _verifyUpdate;
        _changeAdmin(_controller);
        initStage = 1;
    }

    function initSyncCommitteePubkey(bytes memory _syncCommitteePubkeyPart) public {
        require(!initialized, "initialized!");
        require(initStage != 0, "call initialize() first!");

        uint256 pubkeyLength;
        if (initStage % 3 != 0) {
            // initStage 1, 2, 4, 5
            pubkeyLength = 171;
        } else {
            pubkeyLength = 170;
        }
        require(_syncCommitteePubkeyPart.length == pubkeyLength * BLS_PUBKEY_LENGTH, "invalid syncCommitteePubkeyPart");

        bytes32 hash = keccak256(_syncCommitteePubkeyPart);
        require(hash == syncCommitteePubkeyHashes[initStage - 1], "wrong syncCommitteePubkeyPart hash");

        if (initStage < 4) {
            syncCommittees[0].pubkeys = bytes.concat(syncCommittees[0].pubkeys, _syncCommitteePubkeyPart);
        } else {
            syncCommittees[1].pubkeys = bytes.concat(syncCommittees[1].pubkeys, _syncCommitteePubkeyPart);
        }
        if (initStage == 6) {
            initialized = true;
        } else {
            initStage++;
        }
    }

    function updateLightClient(bytes memory _data) external override whenNotPaused {
        require(initialized, "not initialized!");
        require(exeHeaderEndNumber == 0, "previous exe block headers should be updated before update light client");

        Types.LightClientUpdate memory update = abi.decode(_data, (Types.LightClientUpdate));
        require(
            update.attestedHeader.slot >= update.finalizedHeader.slot,
            "invalid attested header and finalized header slot"
        );
        require(update.signatureSlot > update.attestedHeader.slot, "invalid signature slot and attested header slot");
        require(
            update.finalizedHeader.slot > finalizedBeaconHeader.slot,
            "the update finalized slot should be higher than the finalized slot"
        );

        uint256 finalizedPeriod = Helper.compute_sync_committee_period(finalizedBeaconHeader.slot);
        uint256 updatePeriod = Helper.compute_sync_committee_period(update.finalizedHeader.slot);
        require(finalizedPeriod == updatePeriod || finalizedPeriod + 1 == updatePeriod, "unexpected update period");
        require(update.finalityBranch.length == FINALITY_PROOF_SIZE, "invalid finality branch length");
        require(update.executionBranch.length == EXECUTION_PROOF_SIZE, "invalid execution branch length");

        if (finalizedPeriod + 1 == updatePeriod) {
            require(
                update.nextSyncCommitteeBranch.length == NEXT_SYNC_COMMITTEE_PROOF_SIZE,
                "invalid next sync committee branch length"
            );
        }

        if (verifyUpdate) {
            bytes memory encode = abi.encode(
                update,
                finalizedBeaconHeader,
                syncCommittees[curSyncCommitteeIndex],
                syncCommittees[1 - curSyncCommitteeIndex],
                chainId
            );

            uint256 inputLength = encode.length;
            bytes memory result = new bytes(0);
            bool success = false;
            assembly {
                success := staticcall(gas(), 0xe0, add(encode, 32), inputLength, result, 0)
            }
            require(success, "verify light client update failed");
        }

        if (finalizedPeriod + 1 == updatePeriod) {
            syncCommittees[curSyncCommitteeIndex] = update.nextSyncCommittee;
            curSyncCommitteeIndex = 1 - curSyncCommitteeIndex;
        }

        finalizedExeHeaders[update.finalizedExecution.blockNumber] = update.finalizedExecution.blockHash;
        exeHeaderStartNumber = finalizedExeHeaderNumber + 1;
        exeHeaderEndNumber = update.finalizedExecution.blockNumber - 1;
        exeHeaderEndHash = update.finalizedExecution.parentHash;
        finalizedExeHeaderNumber = update.finalizedExecution.blockNumber;
        finalizedBeaconHeader = update.finalizedHeader;

        emit UpdateLightClient(msg.sender, finalizedBeaconHeader.slot, finalizedExeHeaderNumber);
    }

    function updateBlockHeader(bytes memory _blockHeader) external override whenNotPaused {
        require(exeHeaderEndNumber != 0, "no need to update exe headers");

        Types.BlockHeader[] memory headers = abi.decode(_blockHeader, (Types.BlockHeader[]));
        require(headers.length != 0, "invalid headers");
        require(headers.length <= exeHeaderEndNumber - exeHeaderStartNumber + 1, "headers size too big");
        require(headers[0].number >= exeHeaderStartNumber, "invalid start exe header number");
        require(headers[headers.length - 1].number == exeHeaderEndNumber, "invalid end exe header number");

        for (uint256 i = 0; i < headers.length; i++) {
            Types.BlockHeader memory header = headers[headers.length - i - 1];
            require(exeHeaderEndNumber == header.number, "unexpected block number");
            require(exeHeaderEndHash == Helper.getBlockHash(header), "unexpected block parent hash");

            finalizedExeHeaders[exeHeaderEndNumber] = exeHeaderEndHash;
            exeHeaderEndNumber--;
            exeHeaderEndHash = header.parentHash;
        }

        uint256 savedHeaders = finalizedExeHeaderNumber -
            exeHeaderEndNumber +
            exeHeaderStartNumber -
            startExeHeaderNumber;
        uint256 deletedHeaders = 0;
        while (
            savedHeaders > MAX_BLOCK_SAVED &&
            startExeHeaderNumber + 1 < exeHeaderStartNumber &&
            deletedHeaders < MAX_DELETE_COUNT
        ) {
            delete finalizedExeHeaders[startExeHeaderNumber];
            startExeHeaderNumber++;
            savedHeaders--;
            deletedHeaders++;
        }

        if (exeHeaderStartNumber > exeHeaderEndNumber) {
            exeHeaderStartNumber = 0;
            exeHeaderEndNumber = 0;
        }

        emit UpdateBlockHeader(msg.sender, headers[0].number, headers[headers.length - 1].number);
    }

    function verifyProofData(
        bytes memory _receiptProof
    ) external view override returns (bool success, string memory message, bytes memory logs) {
        require(initialized, "contract is not initialized");
        Types.ReceiptProof memory proof = abi.decode(_receiptProof, (Types.ReceiptProof));
        Types.BlockHeader memory header = proof.header;

        // verify block header
        require(header.number <= finalizedExeHeaderNumber, "the execution block is not finalized");
        require(finalizedExeHeaders[header.number] != bytes32(0), "the execution block header is not updated");
        require(finalizedExeHeaders[header.number] == Helper.getBlockHash(header), "invalid execution block header");

        // verify proof
        bytes memory bytesReceipt = Helper.encodeReceipt(proof.txReceipt);
        success = IMPTVerify(mptVerify).verifyTrieProof(
            bytes32(header.receiptsRoot),
            proof.keyIndex,
            proof.proof,
            bytesReceipt
        );

        if (success) {
            if (proof.txReceipt.receiptType != 0) {
                bytesReceipt = Helper.getBytesSlice(bytesReceipt, 1, bytesReceipt.length - 1);
            }
            logs = bytesReceipt.toRlpItem().toList()[3].toRlpBytes();
        } else {
            message = "mpt verify failed";
        }
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProof
    ) external override returns (bool success, string memory message, bytes memory logs) {
        return this.verifyProofData(_receiptProof);
    }

    function clientState() external view override returns (bytes memory) {
        return abi.encode(exeHeaderStartNumber, exeHeaderEndNumber);
    }

    function finalizedState(bytes memory) external pure override returns (bytes memory) {
        return bytes("");
    }

    function headerHeight() external view override returns (uint256) {
        return finalizedBeaconHeader.slot;
    }

    function verifiableHeaderRange() external view override returns (uint256, uint256) {
        if (exeHeaderStartNumber == 0) {
            return (startExeHeaderNumber, finalizedExeHeaderNumber);
        }

        return (startExeHeaderNumber, exeHeaderStartNumber - 1);
    }

    function isVerifiable(uint256 _blockHeight, bytes32 _hash) external view override returns (bool) {
        if (exeHeaderStartNumber == 0) {
            return _blockHeight >= startExeHeaderNumber && _blockHeight <= finalizedExeHeaderNumber;
        }

        return _blockHeight >= startExeHeaderNumber && _blockHeight <= exeHeaderStartNumber - 1;
    }

    function nodeType() external pure override returns (uint256) {
        return 1;
    }

    function notifyLightClient(address _from, bytes memory _data) external override {
        emit ClientNotifySend(_from, block.number, _data);
    }

    function togglePause(bool flag) external onlyOwner returns (bool) {
        if (flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function getBytes(Types.ReceiptProof memory receiptProof) public pure returns (bytes memory) {
        return abi.encode(receiptProof);
    }

    function getHeadersBytes(Types.BlockHeader[] memory _headers) public pure returns (bytes memory) {
        return abi.encode(_headers);
    }

    function getUpdateBytes(Types.LightClientUpdate memory _update) public pure returns (bytes memory) {
        return abi.encode(_update);
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
