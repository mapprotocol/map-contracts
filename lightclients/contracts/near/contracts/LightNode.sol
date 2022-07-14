// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./interface/ILightNode.sol";
import "./lib/Borsh.sol";
import "./lib/NearDecoder.sol";
import "./lib/ProofDecoder.sol";
import "./lib/Ed25519Verify.sol";

contract LightNode is ILightNode {
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;
    using ProofDecoder for Borsh.Data;
    address public owner;
    uint256 public initialized;
    uint256 constant MAX_BLOCK_PRODUCERS = 100;
    struct Epoch {
        bytes32 epochId;
        uint256 numBPs;
        bytes32[MAX_BLOCK_PRODUCERS] keys;
        bytes32[MAX_BLOCK_PRODUCERS / 2] packedStakes;
        uint256 stakeThreshold;
    }
    Epoch[3] public epochs;
    uint256 public curEpoch;
    uint256 public curHeight;

    mapping(uint256 => bytes32) public blockHashes_;
    mapping(uint256 => bytes32) public blockMerkleRoots_;

    bytes public nearProofProducerAccount_;

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }

    constructor() {}

    function initialize(bytes memory nearProofProducerAccount) public {
        require(initialized == 0, "already initialized");

        initialized = 1;

        owner = msg.sender;

        nearProofProducerAccount_ = nearProofProducerAccount;
    }

    function initWithValidators(bytes memory data) public onlyOwner {
        require(
            initialized == 1 && epochs[0].numBPs == 0,
            "Wrong initialization stage"
        );

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.BlockProducer[] memory initialValidators = borsh
            .decodeBlockProducers();
        borsh.done();

        setBlockProducers(initialValidators, epochs[0]);
    }

    // The second part of the initialization -- setting the current head.
    function initWithBlock(bytes memory data) public onlyOwner {
        require(
            initialized == 1 && epochs[0].numBPs != 0,
            "Wrong initialization stage"
        );
        initialized = 2;

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh
            .decodeLightClientBlock();
        borsh.done();

        require(
            nearBlock.next_bps.some,
            "Initialization block must contain next_bps"
        );

        curHeight = nearBlock.inner_lite.height;
        epochs[0].epochId = nearBlock.inner_lite.epoch_id;
        epochs[1].epochId = nearBlock.inner_lite.next_epoch_id;
        blockHashes_[nearBlock.inner_lite.height] = nearBlock.hash;
        blockMerkleRoots_[nearBlock.inner_lite.height] = nearBlock
            .inner_lite
            .block_merkle_root;
        setBlockProducers(nearBlock.next_bps.blockProducers, epochs[1]);
    }

    function verifyProofData(bytes memory _receiptProof)
        external
        view
        override
        returns (bool success, bytes memory logs)
    {
        (uint256 proofBlockHeight, bytes memory proofs) = abi.decode(
            _receiptProof,
            (uint256, bytes)
        );
        Borsh.Data memory borshData = Borsh.from(proofs);

        (success, logs) = _verifyProofData(proofBlockHeight, borshData);
    }

    function updateBlockHeader(bytes memory _blackHeader) external override {
        require(initialized == 2, "Contract is not initialized");
        Borsh.Data memory borsh = Borsh.from(_blackHeader);
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
            if (nearBlock.inner_lite.epoch_id == epochs[curEpoch].epochId) {
                fromNextEpoch = false;
            } else if (
                nearBlock.inner_lite.epoch_id ==
                epochs[(curEpoch + 1) % 3].epochId
            ) {
                fromNextEpoch = true;
            } else {
                revert("Epoch id of the block is not valid");
            }

            // Check that the new block is signed by more than 2/3 of the validators.
            Epoch storage thisEpoch = epochs[
                fromNextEpoch ? (curEpoch + 1) % 3 : curEpoch
            ];
            // Last block in the epoch might contain extra approvals that light client can ignore.
            require(
                nearBlock.approvals_after_next.length >= thisEpoch.numBPs,
                "Approval list is too short"
            );
            // The sum of uint128 values cannot overflow.
            uint256 votedFor = 0;
            for (
                (uint256 i, uint256 cnt) = (0, thisEpoch.numBPs);
                i != cnt;
                ++i
            ) {
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
            require(votedFor > thisEpoch.stakeThreshold, "Too few approvals");

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

            for (
                (uint256 i, uint256 cnt) = (0, thisEpoch.numBPs);
                i < cnt;
                i++
            ) {
                NearDecoder.OptionalSignature memory approval = nearBlock
                    .approvals_after_next[i];
                if (approval.some) {
                    require(
                        Ed25519Verify.checkBlockProducerSignatureInHead(
                            thisEpoch.keys[i],
                            approval.signature.r,
                            approval.signature.s,
                            nearBlock.next_hash,
                            nearBlock.inner_lite.height
                        ),
                        "Invalid Signature"
                    );
                }
            }
            curHeight = nearBlock.inner_lite.height;
            blockHashes_[curHeight] = nearBlock.hash;
            blockMerkleRoots_[curHeight] = nearBlock
                .inner_lite
                .block_merkle_root;
            if (fromNextEpoch) {
                Epoch storage nextEpoch = epochs[(curEpoch + 2) % 3];
                nextEpoch.epochId = nearBlock.inner_lite.next_epoch_id;
                setBlockProducers(nearBlock.next_bps.blockProducers, nextEpoch);
                curEpoch = (curEpoch + 1) % 3;
            }
        }
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
    }

    function _verifyProofData(
        uint256 proofBlockHeight,
        Borsh.Data memory borshData
    ) public view returns (bool success, bytes memory logs) {
        ProofDecoder.ExecutionStatus memory result = parseAndConsumeProof(
            borshData,
            proofBlockHeight
        );

        success = (!result.failed && !result.unknown);

        logs = result.successValue;
    }

    function parseAndConsumeProof(
        Borsh.Data memory borshData,
        uint256 proofBlockHeight
    ) internal view returns (ProofDecoder.ExecutionStatus memory result) {
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borshData
            .decodeFullOutcomeProof();
        borshData.done();

        require(
            proveOutcome(fullOutcomeProof, proofBlockHeight),
            "Proof should be valid"
        );

        require(
            keccak256(
                fullOutcomeProof
                    .outcome_proof
                    .outcome_with_id
                    .outcome
                    .executor_id
            ) == keccak256(nearProofProducerAccount_),
            "Can only withdraw coins from the linked proof producer on Near blockchain"
        );

        result = fullOutcomeProof.outcome_proof.outcome_with_id.outcome.status;
    }

    function proveOutcome(
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof,
        uint256 blockHeight
    ) internal view returns (bool) {
        bytes32 hash = _computeRoot(
            fullOutcomeProof.outcome_proof.outcome_with_id.hash,
            fullOutcomeProof.outcome_proof.proof
        );

        hash = sha256(abi.encodePacked(hash));

        hash = _computeRoot(hash, fullOutcomeProof.outcome_root_proof);

        require(
            hash == fullOutcomeProof.block_header_lite.inner_lite.outcome_root,
            "NearProver: outcome merkle proof is not valid"
        );

        bytes32 expectedBlockMerkleRoot = blockMerkleRoots_[blockHeight];

        require(
            _computeRoot(
                fullOutcomeProof.block_header_lite.hash,
                fullOutcomeProof.block_proof
            ) == expectedBlockMerkleRoot,
            "NearProver: block proof is not valid"
        );

        return true;
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

    function getEcopKeys(uint256 id) public view returns (bytes32[100] memory) {
        bytes32[100] memory keys = epochs[id].keys;

        return keys;
    }
}
