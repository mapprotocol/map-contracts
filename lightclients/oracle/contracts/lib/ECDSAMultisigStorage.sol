// SPDX-License-Identifier: MIT

pragma solidity 0.8.20;

import { EnumerableSet } from '../lib/EnumerableSet.sol';


/**
 * @title ECDSAMultisig Storage
 * @dev derived from https://github.com/solidstate-network/solidstate-solidity(MIT license)
 */
library ECDSAMultisigStorage {
    struct Layout {
        bytes32 version;
        uint256 quorum;
        EnumerableSet.AddressSet signers;
    }

    bytes32 internal constant STORAGE_SLOT =
        keccak256('map.contracts.storage.ECDSAMultisig');

    function layout() internal pure returns (Layout storage l) {
        bytes32 slot = STORAGE_SLOT;
        assembly {
            l.slot := slot
        }
    }
}