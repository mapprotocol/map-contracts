// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "./interface/ILightNode.sol";
import "./lib/Borsh.sol";
import "./lib/NearDecoder.sol";
import "./lib/ProofDecoder.sol";
import "./lib/Ed25519Verify.sol";

contract LightNode is UUPSUpgradeable, Initializable, Pausable, ILightNode {
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;
    using ProofDecoder for Borsh.Data;

    bytes32 private constant zero_byte32 =
        0x0000000000000000000000000000000000000000000000000000000000000000;

    bool public setFirstBlock;
    uint256 constant MAX_BLOCK_PRODUCERS = 100;

    struct Epoch {
        // bytes32 epochId;
        bool init;
        uint256 numBPs;
        bytes32[MAX_BLOCK_PRODUCERS] keys;
        bytes32[MAX_BLOCK_PRODUCERS / 2] packedStakes;
        uint256 stakeThreshold;
    }

    mapping(bytes32 => Epoch) public epochs;

    bytes32 public curEpoch;
    uint256 public curHeight;
    bytes public nearProofProducerAccount_;

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "lightnode :: only admin");
        _;
    }

    event SetNearProofProducerAccount(bytes nearProofProducerAccount);

    event UpdateBlockHeader(bytes32 indexed epochId, uint256 blockHeight);

    //  event SetBlockProducers(bytes32[100] keys);

    constructor() {}

    function initialize(bytes memory nearProofProducerAccount)
        public
        initializer
    {
        _changeAdmin(msg.sender);

        nearProofProducerAccount_ = nearProofProducerAccount;

        emit SetNearProofProducerAccount(nearProofProducerAccount);
    }

    function trigglePause(bool flag) public onlyOwner returns (bool) {
        if (flag) {
            _pause();
        } else {
            _unpause();
        }

        return true;
    }

    function setNearProofProducerAccount_(bytes memory nearProofProducerAccount)
        public
        onlyOwner
    {
        nearProofProducerAccount_ = nearProofProducerAccount;
        emit SetNearProofProducerAccount(nearProofProducerAccount);
    }

    function initWithValidators(bytes memory data) public onlyOwner {
        require(
            !setFirstBlock && epochs[zero_byte32].numBPs == 0,
            "Wrong initialization stage"
        );

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.BlockProducer[] memory initialValidators = borsh
            .decodeBlockProducers();
        borsh.done();

        Epoch storage epoch = epochs[zero_byte32];

        setBlockProducers(initialValidators, epoch);
    }

    // The second part of the initialization -- setting the current head.
    function initWithBlock(bytes memory data) public onlyOwner {
        require(
            !setFirstBlock && epochs[zero_byte32].numBPs != 0,
            "Wrong initialization stage"
        );
        setFirstBlock = true;

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh
            .decodeLightClientBlock();
        borsh.done();

        require(
            nearBlock.next_bps.some,
            "Initialization block must contain next_bps"
        );

        curHeight = nearBlock.inner_lite.height;

        Epoch storage epoch = epochs[nearBlock.inner_lite.epoch_id];

        epoch.numBPs = epochs[zero_byte32].numBPs;

        epoch.keys = epochs[zero_byte32].keys;

        epoch.packedStakes = epochs[zero_byte32].packedStakes;

        epoch.stakeThreshold = epochs[zero_byte32].stakeThreshold;

        epoch.init = true;

        //   delete epochs[zero_byte32];

        curEpoch = nearBlock.inner_lite.epoch_id;

        Epoch storage next = epochs[nearBlock.inner_lite.next_epoch_id];

        next.init = true;

        setBlockProducers(nearBlock.next_bps.blockProducers, next);
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
        (bytes memory _blockHeader, bytes memory proofs) = abi.decode(
            _receiptProof,
            (bytes, bytes)
        );

        Borsh.Data memory borsh = Borsh.from(_blockHeader);
        NearDecoder.LightClientBlock memory nearBlock = borsh
            .decodeLightClientBlock();
        borsh.done();

        (bool _result, string memory _reason) = checkBlockHeader(nearBlock);

        if (!_result) {
            success = false;
            message = _reason;
        } else {
            Borsh.Data memory borshData = Borsh.from(proofs);
            (success, message, logs) = _verifyProofData(
                nearBlock.inner_lite.block_merkle_root,
                borshData
            );
        }
    }

    function updateBlockHeader(bytes memory _blockHeader)
        external
        override
        whenNotPaused
    {
        require(setFirstBlock, "Contract is not initialized");
        Borsh.Data memory borsh = Borsh.from(_blockHeader);
        NearDecoder.LightClientBlock memory nearBlock = borsh
            .decodeLightClientBlock();
        borsh.done();

        unchecked {
            // Check that the new block's height is greater than the current one's.
            require(
                nearBlock.inner_lite.height > curHeight,
                "New block must have higher height"
            );

            // Check that the new block is from the same epoch as the current one, or from the next one.
            bool fromNextEpoch;
            if (nearBlock.inner_lite.epoch_id == curEpoch) {
                fromNextEpoch = false;
            } else {
                fromNextEpoch = true;
            }

            // If the block is from the next epoch, make sure that next_bps is supplied and has a correct hash.
            if (fromNextEpoch) {
                require(
                    nearBlock.next_bps.some,
                    "Next next_bps should not be None"
                );
                require(
                    nearBlock.next_bps.hash ==
                        nearBlock.inner_lite.next_bp_hash,
                    "Hash of block producers does not match"
                );
            }

            (bool result, string memory reason) = checkBlockHeader(nearBlock);

            require(result, reason);

            curHeight = nearBlock.inner_lite.height;

            if (fromNextEpoch) {
                curEpoch = nearBlock.inner_lite.epoch_id;
                Epoch storage nextEpoch = epochs[
                    nearBlock.inner_lite.next_epoch_id
                ];
                nextEpoch.init = true;
                setBlockProducers(nearBlock.next_bps.blockProducers, nextEpoch);
            }

            emit UpdateBlockHeader(curEpoch, curHeight);
        }
    }

    function headerHeight() external view override returns (uint256) {
        return curHeight;
    }

    function checkBlockHeader(NearDecoder.LightClientBlock memory nearBlock)
        internal
        view
        returns (bool, string memory)
    {
        // Check that the new block is signed by more than 2/3 of the validators.
        Epoch storage thisEpoch = epochs[nearBlock.inner_lite.epoch_id];

        if (!thisEpoch.init) {
            return (false, "not init epoch");
        }
        // Last block in the epoch might contain extra approvals that light client can ignore.
        if (nearBlock.approvals_after_next.length < thisEpoch.numBPs) {
            return (false, "Approval list is too short");
        }

        // The sum of uint128 values cannot overflow.
        uint256 votedFor = 0;
        for ((uint256 i, uint256 cnt) = (0, thisEpoch.numBPs); i != cnt; ++i) {
            bytes32 stakes = thisEpoch.packedStakes[i >> 1];
            if (nearBlock.approvals_after_next[i].some) {
                votedFor += uint128(bytes16(stakes));
            }
            if (++i == cnt) {
                break;
            }
            if (nearBlock.approvals_after_next[i].some) {
                votedFor += uint128(uint256(stakes));
            }
        }

        if (votedFor <= thisEpoch.stakeThreshold) {
            return (false, "Too few approvals");
        }

        for ((uint256 i, uint256 cnt) = (0, thisEpoch.numBPs); i < cnt; i++) {
            NearDecoder.OptionalSignature memory approval = nearBlock
                .approvals_after_next[i];
            if (approval.some) {
                bool check = Ed25519Verify.checkBlockProducerSignatureInHead(
                    thisEpoch.keys[i],
                    approval.signature.r,
                    approval.signature.s,
                    nearBlock.next_hash,
                    nearBlock.inner_lite.height
                );
                if (!check) {
                    return (false, "Invalid Signature");
                }
            }
        }

        return (true, "");
    }

    function setBlockProducers(
        NearDecoder.BlockProducer[] memory src,
        Epoch storage epoch
    ) internal {
        uint256 cnt = src.length;
        require(
            cnt <= MAX_BLOCK_PRODUCERS,
            "It is not expected having that many block producers for the provided block"
        );
        epoch.numBPs = cnt;
        unchecked {
            for (uint256 i = 0; i < cnt; i++) {
                epoch.keys[i] = src[i].publicKey.k;
            }
            uint256 totalStake = 0; // Sum of uint128, can't be too big.
            for (uint256 i = 0; i != cnt; ++i) {
                uint128 stake1 = src[i].stake;
                totalStake += stake1;
                if (++i == cnt) {
                    epoch.packedStakes[i >> 1] = bytes32(bytes16(stake1));
                    break;
                }
                uint128 stake2 = src[i].stake;
                totalStake += stake2;
                epoch.packedStakes[i >> 1] = bytes32(
                    uint256(bytes32(bytes16(stake1))) + stake2
                );
            }
            epoch.stakeThreshold = (totalStake * 2) / 3;
        }

        //    emit SetBlockProducers(epoch.keys);
    }

    function _verifyProofData(
        bytes32 block_merkle_root,
        Borsh.Data memory borshData
    )
        public
        view
        returns (
            bool success,
            string memory reason,
            bytes memory logs
        )
    {
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borshData
            .decodeFullOutcomeProof();
        borshData.done();

        (bool _success, string memory _reason) = proveOutcome(
            fullOutcomeProof,
            block_merkle_root
        );

        if (!_success) {
            success = _success;

            reason = _reason;
        } else {
            if (
                keccak256(
                    fullOutcomeProof
                        .outcome_proof
                        .outcome_with_id
                        .outcome
                        .executor_id
                ) != keccak256(nearProofProducerAccount_)
            ) {
                success = false;

                reason = "Can only withdraw coins from the linked proof producer on Near blockchain";
            } else {
                ProofDecoder.ExecutionStatus memory status = fullOutcomeProof
                    .outcome_proof
                    .outcome_with_id
                    .outcome
                    .status;
                success = (!status.failed && !status.unknown);

                if (!success) {
                    reason = "failed or unknow transation";
                }

                logs = abi.encode(
                    fullOutcomeProof.outcome_proof.outcome_with_id.outcome.logs
                );
            }
        }

    }

    function proveOutcome(
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof,
        bytes32 block_merkle_root
    ) internal pure returns (bool, string memory) {
        bytes32 hash = _computeRoot(
            fullOutcomeProof.outcome_proof.outcome_with_id.hash,
            fullOutcomeProof.outcome_proof.proof
        );

        hash = sha256(abi.encodePacked(hash));

        hash = _computeRoot(hash, fullOutcomeProof.outcome_root_proof);

        if (
            hash != fullOutcomeProof.block_header_lite.inner_lite.outcome_root
        ) {
            return (false, "outcome merkle proof is not valid");
        }

        if (
            _computeRoot(
                fullOutcomeProof.block_header_lite.hash,
                fullOutcomeProof.block_proof
            ) != block_merkle_root
        ) {
            return (false, "block proof is not valid");
        }

        return (true, "");
    }

    function _computeRoot(bytes32 node, ProofDecoder.MerklePath memory proof)
        internal
        pure
        returns (bytes32 hash)
    {
        hash = node;
        for (uint256 i = 0; i < proof.items.length; i++) {
            ProofDecoder.MerklePathItem memory item = proof.items[i];
            if (item.direction == 0) {
                hash = sha256(abi.encodePacked(item.hash, hash));
            } else {
                hash = sha256(abi.encodePacked(hash, item.hash));
            }
        }
    }

    function getEcopKeys(bytes32 id) public view returns (bytes32[100] memory) {
        bytes32[100] memory keys = epochs[id].keys;

        return keys;
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
