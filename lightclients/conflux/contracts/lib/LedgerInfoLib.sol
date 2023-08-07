// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "./BCS.sol";
import "./BLS.sol";
import "./Bytes.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

library LedgerInfoLib {

    struct LedgerInfoWithSignatures {
        uint64 epoch;
        uint64 round;
        bytes32 id;                 // block hash
        bytes32 executedStateId;
        uint64 version;
        uint64 timestampUsecs;
        EpochState nextEpochState;  // only available for the last block of epoch
        Decision pivot;             // may be empty for epoch genesis block
        bytes32 consensusDataHash;
        bytes32[] accounts;
        bytes aggregatedSignature;
    }

    struct Decision {
        bytes32 blockHash;
        uint64 height;
    }

    struct EpochState {
        uint64 epoch;
        ValidatorInfo[] validators;
        uint64 quorumVotingPower;
        uint64 totalVotingPower;
        bytes vrfSeed;
    }

    struct ValidatorInfo {
        bytes32 account;
        bytes uncompressedPublicKey; // encoded uncompressed public key in 128 bytes
        bytes vrfPublicKey;
        uint64 votingPower;
    }

    struct Committee {
        uint64 quorumVotingPower;
        mapping (bytes32 => CommitteeMember) members;
        bytes32[] accounts;
        uint256 numAccounts;
    }

    struct CommitteeMember {
        bytes publicKey;
        uint64 votingPower;
    }

    function _validateEpochState(EpochState memory state) private view {
        require(state.epoch > 0, "EpochState: invalid epoch");
        require(state.quorumVotingPower <= state.totalVotingPower, "EpochState: invalid voting power");
        require(state.validators.length > 0, "EpochState: empty validators");

        // IMPORTANT: SECURITY CHECK REQUIRED TO AVOID MALICIOUS MODIFIED UNCOMPRESSED PUBLIC KEY RELAYED.
        // 
        // Validate uncompressed public key, since it is not verified via BLS signature.
        // Basically, use BLS12_G1ADD to verify all public keys are valid (on curve).
        bytes memory input = new bytes(256);
        bytes memory output = new bytes(128);
        for (uint256 i = 0; i < state.validators.length; i += 2) {
            bytes memory key1 = state.validators[i].uncompressedPublicKey;
            require(key1.length == 128, "EpochState: uncompressed public key requires length 128");

            bytes memory key2;
            if (i + 1 >= state.validators.length) {
                key2 = state.validators[0].uncompressedPublicKey;
            } else {
                key2 = state.validators[i + 1].uncompressedPublicKey;
                require(key2.length == 128, "EpochState: uncompressed public key requires length 128");
            }

            Bytes.copy(input, 0, key1);
            Bytes.copy(input, 128, key2);
            BLS.callPrecompile(BLS.PRECOMPILE_BLS12_G1ADD, input, output);
        }
    }

    function updateCommittee(Committee storage committee, EpochState memory state) internal {
        _validateEpochState(state);

        if (committee.quorumVotingPower != state.quorumVotingPower) {
            committee.quorumVotingPower = state.quorumVotingPower;
        }

        // reset mapping
        uint256 oldLen = committee.numAccounts;
        for (uint256 i = 0; i < oldLen; i++) {
            delete committee.members[committee.accounts[i]];
        }

        // update mapping
        uint256 newLen = state.validators.length;
        for (uint256 i = 0; i < newLen; i++) {
            ValidatorInfo memory validator = state.validators[i];
            require(validator.votingPower > 0, "validator voting pow is zero");
            uint256 pubKeyLen = validator.uncompressedPublicKey.length;
            require(pubKeyLen == 96 || pubKeyLen == 128, "invalid BLS public key length");
            committee.members[validator.account] = CommitteeMember(
                validator.uncompressedPublicKey,
                validator.votingPower
            );
        }

        // reset and update accounts array
        if (oldLen == 0) {
            committee.accounts = new bytes32[](newLen);
            committee.numAccounts = newLen;
            oldLen = newLen;
        }

        uint256 len = oldLen < newLen ? oldLen : newLen;
        for (uint256 i = 0; i < len; i++) {
            committee.accounts[i] = state.validators[i].account;
        }

        for (uint256 i = oldLen; i < newLen; i++) {
            committee.accounts.push(state.validators[i].account);
        }

        if (oldLen != newLen) {
            committee.numAccounts = newLen;
        }
    }

    function packSignatures(Committee storage committee, LedgerInfoWithSignatures memory ledgerInfo) internal view returns (
        bytes memory signature, bytes[] memory publicKeys
    ) {
        uint256 numAccounts = ledgerInfo.accounts.length;
        signature = ledgerInfo.aggregatedSignature;
        publicKeys = new bytes[](numAccounts);

        uint256 voted = 0;
        bytes32 lastAccount = 0;

        for (uint256 i = 0; i < numAccounts; i++) {
            // requires in order to avoid duplicated pos account
            bytes32 account = ledgerInfo.accounts[i];
            require(account > lastAccount, "signature accounts not in order");
            lastAccount = account;

            // should be in committee
            CommitteeMember memory member = committee.members[account];
            require(member.votingPower > 0, "validator not in committee");
            voted += member.votingPower;

            publicKeys[i] = member.publicKey;
        }

        require(voted >= committee.quorumVotingPower, "voting power not enough");
    }

    bytes32 private constant BCS_CFX_PREFIX = 0xcd510d1ab583c33b54fa949014601df0664857c18c4cfb228c862dd869df1b62;

    function bcsEncodeLedgerInfo(LedgerInfoWithSignatures memory ledgerInfo) internal pure returns (bytes memory) {
        bytes memory consensusDataHash = abi.encodePacked(ledgerInfo.consensusDataHash);
        bytes memory id = abi.encodePacked(ledgerInfo.id);
        bytes memory executedStateId = abi.encodePacked(ledgerInfo.executedStateId);

        uint256 size = BCS.SIZE_BYTES32 // BCS prefix
            + BCS.SIZE_UINT64 // epoch
            + BCS.SIZE_UINT64 // round
            + BCS.sizeBytes(id)
            + BCS.sizeBytes(executedStateId)
            + BCS.SIZE_UINT64 // version
            + BCS.SIZE_UINT64 // timestampUsecs
            + BCS.SIZE_OPTION + _bcsSize(ledgerInfo.nextEpochState) // Option(nextEpochState)
            + BCS.SIZE_OPTION // Option(pivot)
            + BCS.sizeBytes(consensusDataHash);

        bytes memory pivotBlockHash;
        if (ledgerInfo.pivot.blockHash != 0) {
            pivotBlockHash = bytes(Strings.toHexString(uint256(bytes32(ledgerInfo.pivot.blockHash)), 32));
            size += BCS.SIZE_UINT64; // height
            size += BCS.sizeBytes(pivotBlockHash); // block hash
        }

        Bytes.Builder memory builder = Bytes.newBuilder(size);

        BCS.encodeBytes32(builder, BCS_CFX_PREFIX);
        BCS.encodeUint64(builder, ledgerInfo.epoch);
        BCS.encodeUint64(builder, ledgerInfo.round);
        BCS.encodeBytes(builder, id);
        BCS.encodeBytes(builder, executedStateId);
        BCS.encodeUint64(builder, ledgerInfo.version);
        BCS.encodeUint64(builder, ledgerInfo.timestampUsecs);
        _bcsEncode(builder, ledgerInfo.nextEpochState);

        // pivot
        BCS.encodeOption(builder, ledgerInfo.pivot.blockHash != 0);
        if (ledgerInfo.pivot.blockHash != 0) {
            BCS.encodeUint64(builder, ledgerInfo.pivot.height);
            BCS.encodeBytes(builder, pivotBlockHash);
        }

        BCS.encodeBytes(builder, consensusDataHash);

        return Bytes.seal(builder);
    }

    function _bcsSize(EpochState memory state) private pure returns (uint256 size) {
        if (state.epoch == 0) {
            return 0;
        }

        size += BCS.SIZE_UINT64; // epoch
        size += BCS.SIZE_UINT64; // quorumVotingPower
        size += BCS.SIZE_UINT64; // totalVotingPower

        size += BCS.sizeBytes(state.vrfSeed); // vrf seed

        // validators
        uint256 numValidators = state.validators.length;
        size += BCS.sizeLength(numValidators);
        bytes32 lastAccount = 0;
        for (uint256 i = 0; i < numValidators; i++) {
            // pos account in ASC order
            ValidatorInfo memory validator = state.validators[i];
            require(validator.account > lastAccount, "Validators not in order");
            lastAccount = validator.account;

            // map key: pos account
            size += BCS.SIZE_BYTES32;

            // map value: public key, vrf public key and voting power
            size += 49; // length (1 bytes) + compressed pub key (48 bytes)
            size += BCS.SIZE_OPTION;
            if (validator.vrfPublicKey.length > 0) {
                size += BCS.sizeBytes(validator.vrfPublicKey);
            }
            size += BCS.SIZE_UINT64;
        }
    }

    function _bcsEncode(Bytes.Builder memory builder, EpochState memory state) private pure {
        BCS.encodeOption(builder, state.epoch > 0);

        if (state.epoch == 0) {
            return;
        }

        BCS.encodeUint64(builder, state.epoch);

        uint256 numValidators = state.validators.length;
        BCS.encodeLength(builder, numValidators);
        for (uint256 i = 0; i < numValidators; i++) {
            ValidatorInfo memory validator = state.validators[i];
            BCS.encodeBytes32(builder, validator.account);

            bytes memory compressed = BLS.compressPublicKey(validator.uncompressedPublicKey);
            BCS.encodeBytes(builder, compressed);
            BCS.encodeOption(builder, validator.vrfPublicKey.length > 0);
            if (validator.vrfPublicKey.length > 0) {
                BCS.encodeBytes(builder, validator.vrfPublicKey);
            }
            BCS.encodeUint64(builder, validator.votingPower);
        }

        BCS.encodeUint64(builder, state.quorumVotingPower);
        BCS.encodeUint64(builder, state.totalVotingPower);
        BCS.encodeBytes(builder, state.vrfSeed);
    }

}
