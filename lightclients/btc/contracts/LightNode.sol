// SPDX-License-Identifier: MIT

pragma solidity ^0.8.7;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import  "./interface/ILightNode.sol";
import  "./lib/BytesLib.sol";
import  "./lib/BTCUtils.sol";
import  "./lib/ValidateSPV.sol";
import  "hardhat/console.sol";

contract LightNode is ILightNode,UUPSUpgradeable,Initializable,Pausable,Ownable2Step {

    using BytesLib for bytes;
    using BTCUtils for bytes;

    uint256 public constant DIFFICULTY_ADJUSTMENT_INTERVAL = 2016;
    uint256 public constant CONFIRMATIONS = 6;

    uint256  public constant MAX_BLOCK_NUMBER = 12960;

    uint256 public epochStartTarget;
    uint256 public epochEndTarget;
    uint256 public epochStartTime;
    uint256 public epochEndTime;

    uint256 startHeight;
    uint256 bestHeight;
    bytes32 bestBlock;

    mapping(bytes32 => uint256) public headers;
    mapping(uint256 => bytes32) public finalizedHeaders;

    modifier checkHeader(bytes memory _header){
        require(_header.length == 80, "Invalid block header size");
        _;
    }

    function initialize(
        bytes memory _header,
        uint256 _height
    )
    external
    initializer
    checkHeader(_header)
    {
        require(_height > 0, "Invalid genesis height");
        bytes32 digest = _header.hash256();
        uint256 target = _header.extractTarget();
        uint256 timestamp = _header.extractTimestamp();

        startHeight = _height;
        bestBlock = digest;
        bestHeight = _height;

        finalizedHeaders[_height] = digest;
        headers[digest] = _height;

        epochStartTarget = target;
        epochStartTime = timestamp;
        epochEndTarget = target;
        epochEndTime = timestamp;

        _transferOwnership(tx.origin);
    }


    function verifiableHeaderRange() external view override returns (uint256, uint256){

        return(startHeight,bestHeight);
    }

    function headerHeight() external view override returns (uint256){
        return bestHeight;
    }

    function verifyProofData(bytes memory _proofData) external view override returns (bool) {
        (uint256 height, uint256 index, bytes32 txid, bytes memory header, bytes memory proof) =
        abi.decode(_proofData,(uint256,uint256,bytes32,bytes,bytes));
        // txid must be little endian
        require(txid != 0, "Invalid tx identifier");

        require(height  <= bestHeight, "Insufficient confirmations");

        require(finalizedHeaders[height] == header.hash256(), "Block not found");
        bytes32 root = header.extractMerkleRootLE().toBytes32();
        require(ValidateSPV.prove(txid, root, proof, index), "Incorrect merkle proof");

        return true;
    }

    function updateBlockHeader(bytes memory _headerBytes) external override {
        bytes[] memory updateHeaders = abi.decode(_headerBytes,(bytes[]));
        require(updateHeaders.length >= CONFIRMATIONS,"The block is not final confirmed");
        bytes32 hashCurrBlock = updateHeaders[0].hash256();
        require(headers[hashCurrBlock] == 0, "Block already stored");
        bytes32 hashPrevBlock = updateHeaders[0].extractPrevBlockLE().toBytes32();
        //console.logBytes32(hashPrevBlock);
        require(headers[hashPrevBlock] > 0, "Previous block hash not found");
        uint256 heightPrev = headers[hashPrevBlock];
        for(uint256 i = 0; i < CONFIRMATIONS; i++){

            bytes32 hash = updateHeaders[i].hash256();

            uint256 target = updateHeaders[i].extractTarget();

            require(
                abi.encodePacked(hash).reverseEndianness().bytesToUint() <= target,
                "Insufficient difficulty"
            );

            uint256 tempTime = updateHeaders[i].extractTimestamp();

            uint256 height = heightPrev + i;

            _submitBlockHeader(target,height,tempTime);

        }
        bestHeight = heightPrev + 1;
        bestBlock = hashCurrBlock;

        headers[bestBlock] = bestHeight;
        finalizedHeaders[bestHeight] = bestBlock;

        if(bestHeight > startHeight + MAX_BLOCK_NUMBER){
            uint256 tempHeight = heightPrev - MAX_BLOCK_NUMBER;
            bytes32 tempHash = finalizedHeaders[tempHeight];
            startHeight = tempHeight + 1;
            delete finalizedHeaders[tempHeight];
            delete headers[tempHash];
        }

        emit UpdateBlockHeader(tx.origin, bestHeight);
    }


    function getHeaderBytes(bytes[] memory _headers) external pure returns(bytes memory){
        return abi.encode(_headers);
    }

    function getProofDataBytes(uint256 _height, uint256 _index, bytes32 _txid, bytes memory _header, bytes memory _proof) external pure returns(bytes memory){
        return abi.encode(_height,_index,_txid,_header,_proof);
    }



    function _isPeriodStart(uint256 height) internal pure returns (bool) {
        return height % DIFFICULTY_ADJUSTMENT_INTERVAL == 0;
    }

    function _isPeriodEnd(uint256 height) internal pure returns (bool) {
        return height % DIFFICULTY_ADJUSTMENT_INTERVAL == 2015;
    }


    /**
     * @notice Validates difficulty interval
     * @dev Reverts if previous period invalid
     * @param prevStartTarget Period starting target
     * @param prevStartTime Period starting timestamp
     * @param prevEndTarget Period ending target
     * @param prevEndTime Period ending timestamp
     * @param nextTarget Next period starting target
     * @return True if difficulty level is valid
     */
    function isCorrectDifficultyTarget(
        uint256 prevStartTarget,
        uint256 prevStartTime,
        uint256 prevEndTarget,
        uint256 prevEndTime,
        uint256 nextTarget
    ) internal pure returns (bool) {
        require(
            BTCUtils.calculateDifficulty(prevStartTarget) ==
            BTCUtils.calculateDifficulty(prevEndTarget),
            "Invalid difficulty period"
        );

        uint256 expectedTarget = BTCUtils.retargetAlgorithm(
            prevStartTarget,
            prevStartTime,
            prevEndTime
        );

        return (nextTarget & expectedTarget) == nextTarget;
    }

    function _submitBlockHeader(uint256 _target,uint256 _height,uint256 _extractTimestamp)
    internal
    returns(bool)
    {

        if (_isPeriodStart(_height)) {
            require(
                isCorrectDifficultyTarget(
                    epochStartTarget,
                    epochStartTime,
                    epochEndTarget,
                    epochEndTime,
                    _target
                ),
                "Incorrect difficulty target"
            );

            epochStartTarget = _target;
            epochStartTime = _extractTimestamp;

            delete epochEndTarget;
            delete epochEndTime;
        } else if (_isPeriodEnd(_height)) {
            // only update if end to save gas
            epochEndTarget = _target;
            epochEndTime = _extractTimestamp;
        }

        return true;
    }


    /** UUPS *********************************************************/
    function _authorizeUpgrade(address)
    internal
    view
    onlyOwner
    override {}

    function _transferOwnership(address _newOwner)
    internal
    virtual
    override
    {
        super._transferOwnership(_newOwner);
        _changeAdmin(_newOwner);
    }

    function getAdmin() external view returns (address){
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
