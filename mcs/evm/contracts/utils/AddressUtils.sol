// SPDX-License-Identifier: MIT

library AddressUtils {
    function fromBytes(bytes memory bys) internal pure returns (address addr){
        assembly {
            addr := mload(add(bys, 20))
        }
    }

    function toBytes(address self) internal pure returns (bytes memory b) {
        b = abi.encodePacked(self);
    }
}