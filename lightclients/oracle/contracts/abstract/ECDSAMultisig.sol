// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {EnumerableSet} from "../lib/EnumerableSet.sol";
import {ECDSAMultisigStorage} from "../lib/ECDSAMultisigStorage.sol";

/**
 * @title ECDSAMultisig
 * @dev derived from https://github.com/solidstate-network/solidstate-solidity(MIT license)
 */
abstract contract ECDSAMultisig {
    error ECDSAMultisig_AddSignerFailed();
    error ECDSAMultisig_QuorumNotReached();
    error ECDSAMultisig_RemoveSignerFailed();
    error ECDSAMultisig_SignerLimitReached();
    error ECDSAMultisig_SignerAlreadySigned();
    error ECDSAMultisig_InsufficientSigners();
    error ECDSAMultisig_MessageValueMismatch();
    error ECDSAMultisig_RecoveredSignerNotAuthorized();

    using ECDSA for bytes32;
    using EnumerableSet for EnumerableSet.AddressSet;

    function _setQuorum(uint256 quorum) internal {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();

        if (quorum > l.signers.length()) revert ECDSAMultisig_InsufficientSigners();
        l.quorum = quorum;
    }

    function _setVersion(bytes32 version) internal {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();
        l.version = version;
    }

    function _isSigner(address account) internal view returns (bool) {
        return ECDSAMultisigStorage.layout().signers.contains(account);
    }

    function _addSigner(address account) internal {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();

        if (l.signers.length() >= 256) revert ECDSAMultisig_SignerLimitReached();
        if (!l.signers.add(account)) revert ECDSAMultisig_AddSignerFailed();
    }

    function _removeSigner(address account) internal {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();

        if (l.quorum > l.signers.length() - 1) revert ECDSAMultisig_InsufficientSigners();
        if (!l.signers.remove(account)) revert ECDSAMultisig_RemoveSignerFailed();
    }

    function _multisigInfo() internal view virtual returns (bytes32 version, uint256 quorum, address[] memory singers) {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();
        version = l.version;
        quorum = l.quorum;
        singers = l.signers.toArray();
    }

    function _signers() internal view returns (address[] memory singers) {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();
        return l.signers.toArray();
    }

    function _version() internal view returns (bytes32) {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();
        return l.version;
    }

    function _quorum() internal view returns (uint256) {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();
        return l.quorum;
    }

    function _verifySignatures(
        bytes32 root,
        uint256 blockNum,
        uint256 chainId,
        bytes[] memory signatures
    ) internal view virtual {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();

        if (l.quorum > signatures.length) revert ECDSAMultisig_QuorumNotReached();

        uint256 signerBitmap;

        unchecked {
            for (uint256 i; i < signatures.length; i++) {
                address signer = keccak256(abi.encodePacked(root, l.version, blockNum, chainId))
                    .toEthSignedMessageHash()
                    .recover(signatures[i]);

                uint256 index = l.signers.indexOf(signer);
                // if signer noexist index overflow unchecked{0 - 1}
                if (index >= 256) revert ECDSAMultisig_RecoveredSignerNotAuthorized();

                uint256 shift = 1 << index;

                if (signerBitmap & shift != 0) revert ECDSAMultisig_SignerAlreadySigned();

                signerBitmap |= shift;
            }
        }
    }

    function _verifySignature(
        bytes32 root,
        uint256 blockNum,
        uint256 chainId,
        bytes memory signature
    ) internal view returns (address signer) {
        ECDSAMultisigStorage.Layout storage l = ECDSAMultisigStorage.layout();
        signer = keccak256(abi.encodePacked(root, l.version, blockNum, chainId)).toEthSignedMessageHash().recover(
            signature
        );

        if (!_isSigner(signer)) revert ECDSAMultisig_RecoveredSignerNotAuthorized();
    }
}
