// SPDX-License-Identifier: MIT

pragma solidity >0.8.0;

import "../interface/IBLSPoint.sol";

contract BlsCode is IBLSPoint{

    struct G1Bytes {
        bytes32 x;
        bytes32 y;
    }

    struct G2Bytes {
        bytes32 xr;
        bytes32 xi;
        bytes32 yr;
        bytes32 yi;
    }

    function bytesToUint(bytes32 b) public pure returns (uint){
        uint number;
        for(uint i= 0; i<b.length; i++){
            number = number + uint8(b[i])*(2**(8*(b.length-(i+1))));
        }
        return  number;
    }

    function uintToBytes(uint x) public pure returns (bytes memory) {
        return abi.encodePacked(x);
    }

    function decodeG1Bytes(bytes memory g1Bytes) public pure returns (G1Bytes memory){
        bytes32  x;
        bytes32  y;
        assembly {
            x := mload(add(g1Bytes, 32))
            y := mload(add(g1Bytes, 64))
        }
        return G1Bytes(x,y);
    }

    function decodeG1(bytes memory g1Bytes) public pure returns (G1 memory){
        G1Bytes memory g1b = decodeG1Bytes(g1Bytes);
        return  G1(
            bytesToUint(g1b.x),
            bytesToUint(g1b.y)
        );
    }

    function decodeG2Bytes(bytes memory g1Bytes) public pure returns (G2Bytes memory){
        bytes32 xr;
        bytes32 xi;
        bytes32 yr;
        bytes32 yi;
        assembly {
            xi := mload(add(g1Bytes, 32))
            xr := mload(add(g1Bytes, 64))
            yi := mload(add(g1Bytes, 96))
            yr := mload(add(g1Bytes, 128))
        }
        return G2Bytes(xr, xi,yr,yi);
    }

    function decodeG2(bytes memory g2Bytes) public pure returns (G2 memory){
        G2Bytes memory g2b = decodeG2Bytes(g2Bytes);
        return G2(
            bytesToUint(g2b.xi),
            bytesToUint(g2b.xr),
            bytesToUint(g2b.yi),
            bytesToUint(g2b.yr)
        );
    }

    function encodeG1(G1 memory g1) public pure returns (bytes memory){
        return abi.encodePacked(g1.x,g1.y);
    }

    function encodeG2(G2 memory g2) public pure returns (bytes memory){
        return abi.encodePacked(g2.xi,g2.xr,g2.yi,g2.yr);
    }
}