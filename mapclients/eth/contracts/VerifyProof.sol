// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;


import "./lib/MPT.sol";

contract VerifyProof  {

    using MPT for MPT.MerkleProof;




    constructor()  {}


    function verifyTrieProof(bytes32 hash, bytes memory _expectedValue, bytes[] memory proofs,bytes memory _key) pure public returns (bool success) {

        MPT.MerkleProof memory mp;
        mp.expectedRoot =  hash;
        mp.key = _key;
        mp.proof = proofs;
        mp.keyIndex = 0;
        mp.proofIndex = 0;
        mp.expectedValue = _expectedValue;


        success =MPT.verifyTrieProof(mp);


    }


}