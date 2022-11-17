// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./interface/IMPTVerify.sol";
import "./lib/RLPReader.sol";
import "./lib/RLPEncode.sol";

import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint256 public constant EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
    uint256 public constant MIN_SYNC_COMMITTEE_PARTICIPANTS = 1;
    uint256 public constant SLOTS_PER_EPOCH = 32;
    uint256 public constant FINALITY_TREE_DEPTH = 6;
    uint256 public constant EXECUTION_PROOF_SIZE = 8;
    uint256 public constant BLS_PUBKEY_LENGTH = 48;

    uint64 public chainId;
    address public mptVerify;
    bool public initialized;

    mapping(uint256 => bytes32) public finalizedExeHeaders;
    uint256 public finalizedExeHeaderNumber;
    ExeHeaderUpdateInfo public exeHeaderUpdateInfo;
    BeaconBlockHeader public finalizedBeaconHeader;
    uint256 private _startExeHeaderNumber;

    SyncCommittee[2] public syncCommittees;
    uint64 private _curSyncCommitteeIndex;
    bytes32[] private _syncCommitteePubkeyHashes;
    bool verifyUpdate;
    uint64 public initStage;

    event UpdateLightClientEvent(LightClientUpdate update);
    event UpdateExeBlockHeader(uint256 start, uint256 end);

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    function initialize(
        uint64 _chainId,
        address _controller,
        address _mptVerify,
        BeaconBlockHeader memory _finalizedBeaconHeader,
        uint256 _finalizedExeHeaderNumber,
        bytes32 _finalizedExeHeaderHash,
        bytes memory curSyncCommitteeAggPubKey,
        bytes memory nextSyncCommitteeAggPubKey,
        bytes32[] memory syncCommitteePubkeyHashes, // divide 512 pubkeys into 3 parts: 171 + 171 + 170
        bool _verifyUpdate
    ) public initializer {
        require(!initialized, "contract is initialized!");
        require(_controller != address(0), "invalid controller address");
        require(_mptVerify != address(0), "invalid mptVerify address");
        require(syncCommitteePubkeyHashes.length == 6, "invalid syncCommitteePubkeyHashes length");
        for (uint256 i = 0; i < syncCommitteePubkeyHashes.length; i++) {
            require(syncCommitteePubkeyHashes[i] != bytes32(0), "invalid syncCommitteePubkeyHashes item");
        }
        require(curSyncCommitteeAggPubKey.length == BLS_PUBKEY_LENGTH, "invalid curSyncCommitteeAggPubKey");
        require(nextSyncCommitteeAggPubKey.length == BLS_PUBKEY_LENGTH, "invalid nextSyncCommitteeAggPubKey");

        chainId = _chainId;
        mptVerify = _mptVerify;
        finalizedBeaconHeader = _finalizedBeaconHeader;
        finalizedExeHeaderNumber = _finalizedExeHeaderNumber;
        finalizedExeHeaders[_finalizedExeHeaderNumber] = _finalizedExeHeaderHash;
        _startExeHeaderNumber = _finalizedExeHeaderNumber;

        syncCommittees[0].aggregatePubkey = curSyncCommitteeAggPubKey;
        syncCommittees[1].aggregatePubkey = nextSyncCommitteeAggPubKey;
        _syncCommitteePubkeyHashes = syncCommitteePubkeyHashes;
        _curSyncCommitteeIndex = 0;

        verifyUpdate = _verifyUpdate;
        _changeAdmin(_controller);
        initStage = 1;
    }


    function initSyncCommitteePubkey(
        bytes memory syncCommitteePubkeyPart
    ) public {
        require(!initialized, "contract is initialized!");
        uint256 pubkeyLength;

        if (initStage % 3 != 0) {// initStage 1, 2, 4, 5
            pubkeyLength = 171;
        } else {
            pubkeyLength = 170;
        }
        require(syncCommitteePubkeyPart.length == pubkeyLength * BLS_PUBKEY_LENGTH, "invalid syncCommitteePubkeyPart");

        bytes32 hash = keccak256(syncCommitteePubkeyPart);
        require(hash == _syncCommitteePubkeyHashes[initStage - 1], "wrong syncCommitteePubkeyPart hash");

        if (initStage < 4) {
            syncCommittees[0].pubkeys = bytes.concat(syncCommittees[0].pubkeys, syncCommitteePubkeyPart);
        } else {
            syncCommittees[1].pubkeys = bytes.concat(syncCommittees[1].pubkeys, syncCommitteePubkeyPart);
        }
        if (initStage == 6) {
            initialized = true;
        } else {
            initStage++;
        }
    }

    function updateLightClient(LightClientUpdate memory update)
    external
    override
    whenNotPaused
    {
        require(initialized, "contract is not initialized!");
        require(exeHeaderUpdateInfo.endNumber == 0, "previous exe block headers should be updated before update light client");
        require(update.attestedHeader.slot >= update.finalizedHeader.slot, "invalid attested header and finalized header slot");
        require(update.signatureSlot > update.attestedHeader.slot, "invalid signature slot and attested header slot");
        require(update.finalizedHeader.slot > finalizedBeaconHeader.slot, "the update finalized slot should be higher than the finalized slot");

        uint256 finalizedPeriod = compute_sync_committee_period(finalizedBeaconHeader.slot);
        uint256 updatePeriod = compute_sync_committee_period(update.finalizedHeader.slot);
        require(finalizedPeriod == updatePeriod || finalizedPeriod + 1 == updatePeriod, "unexpected update period");
        require(update.finalityBranch.length == FINALITY_TREE_DEPTH, "invalid finality branch length");

        if (finalizedPeriod + 1 == updatePeriod) {
            require(update.exeFinalityBranch.length == EXECUTION_PROOF_SIZE, "invalid execution finality branch length");
        }

        if (verifyUpdate) {
            bytes memory encode = encodeUpdateAndState(update);
            uint256 inputLength = encode.length;
            bytes memory result = new bytes(0);
            bool success = false;
            assembly {
                success := staticcall(gas(), 31, encode, inputLength, result, 0)
            }
            require(success, "verify light client update failed");
        }

        if (finalizedPeriod + 1 == updatePeriod) {
            syncCommittees[_curSyncCommitteeIndex] = update.nextSyncCommittee;
            _curSyncCommitteeIndex = 1 - _curSyncCommitteeIndex;
        }

        finalizedExeHeaders[update.finalizedExeHeader.number] = getBlockHash(update.finalizedExeHeader);
        exeHeaderUpdateInfo.startNumber = finalizedExeHeaderNumber + 1;
        exeHeaderUpdateInfo.endNumber = update.finalizedExeHeader.number - 1;
        exeHeaderUpdateInfo.endHash = bytesToBytes32(update.finalizedExeHeader.parentHash, 0);
        finalizedExeHeaderNumber = update.finalizedExeHeader.number;
        finalizedBeaconHeader = update.finalizedHeader;

        emit UpdateLightClient(msg.sender, finalizedExeHeaderNumber);
    }

    function updateExeBlockHeaders(BlockHeader[] memory headers)
    external
    {
        require(headers.length != 0, "invalid headers");
        require(exeHeaderUpdateInfo.endNumber != 0, "no need to update exe headers");
        require(headers.length <= exeHeaderUpdateInfo.endNumber - exeHeaderUpdateInfo.startNumber + 1, "headers size too big");
        require(headers[0].number >= exeHeaderUpdateInfo.startNumber, "invalid start exe header number");
        require(headers[headers.length - 1].number == exeHeaderUpdateInfo.endNumber, "invalid end exe header number");

        for (uint256 i = 0; i < headers.length; i++) {
            BlockHeader memory header = headers[headers.length - i - 1];
            require(exeHeaderUpdateInfo.endNumber == header.number, "unexpected block number");
            require(exeHeaderUpdateInfo.endHash == getBlockHash(header), "unexpected block parent hash");

            finalizedExeHeaders[exeHeaderUpdateInfo.endNumber] = exeHeaderUpdateInfo.endHash;
            exeHeaderUpdateInfo.endNumber--;
            exeHeaderUpdateInfo.endHash = bytesToBytes32(header.parentHash, 0);
        }

        if (exeHeaderUpdateInfo.startNumber > exeHeaderUpdateInfo.endNumber) {
            exeHeaderUpdateInfo.startNumber = 0;
            exeHeaderUpdateInfo.endNumber = 0;
        }

        emit UpdateExeBlockHeader(headers[0].number, headers[headers.length - 1].number);
    }


    function verifyProofData(bytes memory receiptProof)
    external
    view
    override
    returns (
        bool success,
        string memory message,
        bytes memory logs
    )
    {
        require(initialized, "contract is not initialized");
        ReceiptProof memory proof = abi.decode(receiptProof, (ReceiptProof));
        BlockHeader memory header = proof.header;
        bytes32 hash = getBlockHash(header);

        // verify block header
        require(header.number <= finalizedExeHeaderNumber, "the execution block is not finalized");
        require(finalizedExeHeaders[header.number] != bytes32(0), "the execution block header is not updated");
        require(finalizedExeHeaders[header.number] == hash, "invalid execution block header");

        // verify proof
        bytes memory bytesReceipt = encodeReceipt(proof.txReceipt);
        success = IMPTVerify(mptVerify).verifyTrieProof(
            bytes32(proof.header.receiptsRoot),
            proof.keyIndex,
            proof.proof,
            bytesReceipt
        );

        if (success) {
            if (proof.txReceipt.receiptType != 0) {
                bytesReceipt = getBytesSlice(bytesReceipt, 1, bytesReceipt.length - 1);
            }
            logs = bytesReceipt.toRlpItem().safeGetItemByIndex(3).toBytes();
        } else {
            message = "mpt verify failed";
        }
    }

    function verifiableHeaderRange() external view returns (uint256, uint256) {
        if (exeHeaderUpdateInfo.startNumber == 0) {
            return (_startExeHeaderNumber, finalizedExeHeaderNumber);
        }

        return (_startExeHeaderNumber, exeHeaderUpdateInfo.startNumber - 1);
    }

    function togglePause(bool flag) external onlyOwner returns (bool) {
        if (flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function encodeUpdateAndState(LightClientUpdate memory update)
    internal
    view
    returns (bytes memory)
    {
        LightClientState memory state = LightClientState(
            finalizedBeaconHeader,
            syncCommittees[_curSyncCommitteeIndex],
            syncCommittees[1 - _curSyncCommitteeIndex],
            chainId
        );
        LightClientVerify memory verify = LightClientVerify(update, state);

        return abi.encode(verify);
    }

    function finalizedSlot() external view override returns (uint256) {
        return finalizedBeaconHeader.slot;
    }

    function compute_epoch_at_slot(uint256 slot) internal pure returns (uint256) {
        return slot / SLOTS_PER_EPOCH;
    }

    function compute_sync_committee_period(uint256 slot) internal pure returns (uint256){
        return compute_epoch_at_slot(slot) / EPOCHS_PER_SYNC_COMMITTEE_PERIOD;
    }

    function calcSyncCommitteeHash(SyncCommittee memory syncCommittee)
    public
    pure
    returns (bytes32)
    {
        return keccak256(abi.encodePacked(syncCommittee.pubkeys, syncCommittee.aggregatePubkey));
    }

    function getBlockHash(BlockHeader memory header)
    public
    pure
    returns (bytes32)
    {
        bytes[] memory list = new bytes[](16);
        list[0] = RLPEncode.encodeBytes(header.parentHash);
        list[1] = RLPEncode.encodeBytes(header.sha3Uncles);
        list[2] = RLPEncode.encodeAddress(header.miner);
        list[3] = RLPEncode.encodeBytes(header.stateRoot);
        list[4] = RLPEncode.encodeBytes(header.transactionsRoot);
        list[5] = RLPEncode.encodeBytes(header.receiptsRoot);
        list[6] = RLPEncode.encodeBytes(header.logsBloom);
        list[7] = RLPEncode.encodeUint(header.difficulty);
        list[8] = RLPEncode.encodeUint(header.number);
        list[9] = RLPEncode.encodeUint(header.gasLimit);
        list[10] = RLPEncode.encodeUint(header.gasUsed);
        list[11] = RLPEncode.encodeUint(header.timestamp);
        list[12] = RLPEncode.encodeBytes(header.extraData);
        list[13] = RLPEncode.encodeBytes(header.mixHash);
        list[14] = RLPEncode.encodeBytes(header.nonce);
        list[15] = RLPEncode.encodeUint(header.baseFeePerGas);
        return keccak256(RLPEncode.encodeList(list));
    }

    function encodeReceipt(TxReceipt memory _txReceipt)
    public
    pure
    returns (bytes memory output)
    {
        bytes[] memory list = new bytes[](4);
        list[0] = RLPEncode.encodeBytes(_txReceipt.postStateOrStatus);
        list[1] = RLPEncode.encodeUint(_txReceipt.cumulativeGasUsed);
        list[2] = RLPEncode.encodeBytes(_txReceipt.bloom);
        bytes[] memory listLog = new bytes[](_txReceipt.logs.length);
        bytes[] memory loglist = new bytes[](3);
        for (uint256 j = 0; j < _txReceipt.logs.length; j++) {
            loglist[0] = RLPEncode.encodeAddress(_txReceipt.logs[j].addr);
            bytes[] memory loglist1 = new bytes[](
                _txReceipt.logs[j].topics.length
            );
            for (uint256 i = 0; i < _txReceipt.logs[j].topics.length; i++) {
                loglist1[i] = RLPEncode.encodeBytes(
                    _txReceipt.logs[j].topics[i]
                );
            }
            loglist[1] = RLPEncode.encodeList(loglist1);
            loglist[2] = RLPEncode.encodeBytes(_txReceipt.logs[j].data);
            bytes memory logBytes = RLPEncode.encodeList(loglist);
            listLog[j] = logBytes;
        }
        list[3] = RLPEncode.encodeList(listLog);
        if (_txReceipt.receiptType == 0) {
            output = RLPEncode.encodeList(list);
        } else {
            bytes memory tempType = abi.encode(_txReceipt.receiptType);
            bytes1 tip = tempType[31];
            bytes memory temp = RLPEncode.encodeList(list);
            output = abi.encodePacked(tip, temp);
        }
    }

    function getBytes(ReceiptProof memory receiptProof) public pure returns (bytes memory){
        return abi.encode(receiptProof);
    }

    function bytesToBytes32(bytes memory b, uint256 offset)
    private
    pure
    returns (bytes32) {
        bytes32 out;

        for (uint256 i = 0; i < 32; i++) {
            out |= bytes32(b[offset + i] & 0xFF) >> (i * 8);
        }
        return out;
    }

    function getBytesSlice(bytes memory b, uint256 start, uint256 length)
    private
    pure
    returns (bytes memory) {
        bytes memory out = new bytes(length);

        for (uint256 i = 0; i < length; i++) {
            out[i] = b[start + i];
        }

        return out;
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
