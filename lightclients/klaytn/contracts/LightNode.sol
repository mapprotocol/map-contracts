// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "./lib/RLPReader.sol";
import "./interface/IVerifyTool.sol";
import "./interface/ILightNode.sol";
import "hardhat/console.sol";

contract LightNode is UUPSUpgradeable, Initializable, ILightNode, Ownable2Step {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint8   constant MSG_COMMIT = 2;
    uint256 constant MAX_VALIDATORS_SIZE = 2160;
    uint256 constant CHANGE_VALIDATORS_SIZE = 3600;
    uint256 constant RLP_INDEX = 3;
    bytes32 constant ADD_VALIDATOR = 0x9faa13f6fa6f531607d2fc3a8956aa591b138a5e2690cba6cd54f56e7b2324c8;
    bytes32 constant REMOVE_VALIDATOR = 0x3e9698b37f61d5135393cc4891dd22b1a42d2d350e5d561bcd6967bf75589818;

    uint256 public headerHeight;
    uint256 public validatorIdx;
    uint256 public startHeight;
    uint256 public tempBlockHeight;
    IVerifyTool public verifyTool;
    mapping(uint256 => Validator) public extendValidator;
    mapping(uint256 => uint256) public extendList;
    Validator[MAX_VALIDATORS_SIZE] public validators;

    uint256 public committeeSize;

    struct Validator {
        address[] validators;
        uint256 headerHeight;
    }


    function initialize(
        address[] memory _validators,
        uint256 _headerHeight,
        address _verifyTool
    )
    external
    override
    initializer
    checkAddress(_verifyTool)
    checkMultipleAddress(_validators)
    {
        Validator memory _validator = Validator({
        validators : _validators,
        headerHeight : _headerHeight
        });
        headerHeight = _headerHeight;
        validatorIdx = _getValidatorIndex(headerHeight);
        validators[validatorIdx] = _validator;
        startHeight = _headerHeight;
        verifyTool = IVerifyTool(_verifyTool);
        committeeSize = 31;

        _transferOwnership(tx.origin);
    }


    modifier checkAddress(address _address){
        require(_address != address(0), "address is zero");
        _;
    }

    modifier checkMultipleAddress(address[] memory _addressArray){
        for (uint i = 0; i < _addressArray.length; i++) {
            require(_addressArray[i] != address(0), "address have zero");
        }
        _;
    }


    function verifyProofData(bytes memory _receiptProof)
    external
    view
    override
    returns (bool success,
        string memory message,
        bytes memory logs)
    {
        ReceiptProof memory receiptProof = abi.decode(_receiptProof, (ReceiptProof));

        if (receiptProof.deriveSha == DeriveShaOriginal.DeriveShaConcat){
            ReceiptProofConcat memory proof = abi.decode(receiptProof.proof,(ReceiptProofConcat));
            BlockHeader memory header = proof.header;
            (success, ) = checkBlockHeader(header,true);
            if(!success){
                message = "DeriveShaConcat header verify failed";
                return(success,message,logs);
            }
            success = verifyTool.checkReceiptsConcat(proof.receipts, (bytes32)(header.receiptsRoot));
            if (success) {
                bytes memory bytesReceipt = proof.receipts[proof.logIndex];
                RLPReader.RLPItem memory logsItem = bytesReceipt.toRlpItem().safeGetItemByIndex(RLP_INDEX);
                logs = RLPReader.toRlpBytes(logsItem);
                message = "DeriveShaConcat mpt verify success";
                return(success,message,logs);
            }else{
                message = "DeriveShaConcat mpt verify failed";
                return(success,message,logs);
            }
        } else if (receiptProof.deriveSha == DeriveShaOriginal.DeriveShaOriginal) {
            ReceiptProofOriginal memory proof = abi.decode(receiptProof.proof,(ReceiptProofOriginal));
            (success, ) = checkBlockHeader(proof.header,true);
            if(!success){
                message = "DeriveShaOriginal header verify failed";
                return(success,message,logs);
            }
            (success,logs) = verifyTool.checkReceiptsOriginal(proof);
            if (success) {
                message = "DeriveShaOriginal mpt verify success";
                return(success,message,logs);
            }else{
                message = "DeriveShaOriginal mpt verify failed";
                return(success,message,logs);
            }
        }else{
            message = "mpt verify failed";
            success = false;
            return(success,message,logs);
        }
    }

    function updateBlockHeader(bytes memory _blockHeaders)
    external
    override
    {
        BlockHeader[] memory _headers = abi.decode(
            _blockHeaders, (BlockHeader[]));

        require(_headers[0].number > headerHeight, "height error");
        if(_headers[0].number % CHANGE_VALIDATORS_SIZE > 0) {

            _updateBlockHeaderChange(_headers);
        }else{

            for (uint256 i = 0; i < _headers.length; i++) {
                require(_headers[i].number == headerHeight + CHANGE_VALIDATORS_SIZE, "height epoch error");
                BlockHeader memory bh = _headers[i];
                (bool success, ExtraData memory data) = checkBlockHeader(bh, false);
                require(success, "header verify fail");

                validatorIdx = _getValidatorIndex(bh.number);
                Validator memory tempValidators = validators[validatorIdx];

                while(extendList[tempValidators.headerHeight] > 0){
                    uint256 tempHeight = _getRemoveExtendHeight(tempValidators.headerHeight);
                    uint256 trueHeight = _getTrueHeight(tempValidators.headerHeight,tempHeight);
                    delete extendValidator[tempHeight];
                    delete extendList[trueHeight];
                }
                Validator memory v = Validator({
                validators : data.validators,
                headerHeight : bh.number
                });
                validators[validatorIdx] = v;
                headerHeight = bh.number;
                emit UpdateBlockHeader(msg.sender,headerHeight);
            }
        }

    }

    function verifiableHeaderRange()
    external
    override
    view
    returns (uint256 start, uint256 end){
        return (_getStartValidatorHeight(), _getEndValidatorHeight());
    }


    function getBytes(ReceiptProofOriginal memory _proof)
    external
    pure
    returns (bytes memory)
    {
        bytes memory proof = abi.encode(_proof);

        ReceiptProof memory receiptProof = ReceiptProof(proof,DeriveShaOriginal.DeriveShaOriginal);

        return abi.encode(receiptProof);
    }

    function getHeadersBytes(BlockHeader[] memory _blockHeaders)
    external
    pure
    returns (bytes memory)
    {
        return abi.encode(_blockHeaders);
    }

    function setCommitteeSize(uint256 _committeeSize) external onlyOwner {
        require(_committeeSize > 0,"Committee size error");
        committeeSize = _committeeSize;

    }

    function _updateBlockHeaderChange(BlockHeader[] memory _blockHeaders)
    internal
    {
        BlockHeader memory header0 = _blockHeaders[0];
        BlockHeader memory header1 = _blockHeaders[1];
        require(header0.voteData.length > 0,"The extension update is not satisfied");
        require(header0.number + 1 == header1.number, "Synchronous height error");
        require(header0.number >= tempBlockHeight ,"updata height error");

        (bool success, ExtraData memory header1Extra) = checkBlockHeader(header1, true);
        (bool hearderTag0,) = checkBlockHeader(header0, true);
        require(success && hearderTag0, "header change verify fail");

        Vote memory vote = verifyTool.decodeVote(_blockHeaders[0].voteData);
        require( vote.value.length % 20 == 0,"address error");
        address[] memory newValidator = verifyTool.bytesToAddressArray(vote.value);
        bool success1;
        if(keccak256(vote.key) == ADD_VALIDATOR){
            for(uint256 i = 0; i < newValidator.length; i++){
                success1  = _checkCommittedAddress(header1Extra.validators,newValidator[i]);
                require(success1,"ADD_VALIDATOR error");
            }
        }else if (keccak256(vote.key) == REMOVE_VALIDATOR){
            for(uint256 i = 0; i < newValidator.length; i++){
                success1  = _checkCommittedAddress(header1Extra.validators,newValidator[i]);
                require(!success1 ,"REMOVE_VALIDATOR error");
            }
        }else{
            require(false,"Not the expected instruction");
        }

        Validator memory v = Validator({
        validators : header1Extra.validators,
        headerHeight : header1.number
        });
        extendValidator[header1.number] = v;
        startHeight = _getBlockHeightList(header1.number,true);
        extendList[startHeight] = header1.number;
        tempBlockHeight = header1.number;
        emit UpdateBlockHeader(msg.sender,tempBlockHeight);
    }

    function _getBlockHeightList(uint256 _height,bool _tag)
    internal
    view
    returns(uint256 truetHeight)
    {
        uint256 opochBlockHeight = (_height / CHANGE_VALIDATORS_SIZE) * CHANGE_VALIDATORS_SIZE;
        if(extendList[opochBlockHeight] > 0){
            if(!_tag) {
                _height = _height + CHANGE_VALIDATORS_SIZE;
            }
            if(_height >= tempBlockHeight){
                truetHeight = tempBlockHeight;
            }else{
                truetHeight = _getTrueHeight(opochBlockHeight,_height);
            }
        }else{
            truetHeight = opochBlockHeight;
        }
    }

    function _getTrueHeight(uint256 _height,uint256 _verifyHeight)
    internal
    view
    returns(uint256){
        if(extendList[_height] >= _verifyHeight){
            return _height;
        }else {
           return _getTrueHeight(extendList[_height],_verifyHeight);
        }
    }

    function _getRemoveExtendHeight(uint256 _height)
    internal
    view
    returns(uint256){
        if(extendList[_height] == 0){
            return _height;
        }else{
            return _getRemoveExtendHeight(extendList[_height]);
        }
    }

    function checkBlockHeader(BlockHeader memory _header,bool _tag)
    internal
    view
    returns (bool, ExtraData memory)
    {

        bool success = verifyTool.checkHeaderParam(_header);

        require(success, "header param error");

        (bytes memory extHead, ExtraData memory ext) = verifyTool.decodeHeaderExtraData(_header.extraData);
        (bytes memory extraNoSeal, bytes memory seal) = verifyTool.getRemoveSealExtraData(ext, extHead, false);
        (bytes memory extra,) = verifyTool.getRemoveSealExtraData(ext, extHead, true);
        (bytes32 hash,bytes32 signerHash) = verifyTool.getBlockNewHash(_header, extra,extraNoSeal);

        address signer = verifyTool.recoverSigner(seal, keccak256(abi.encodePacked(signerHash)));

        uint num = _header.number;

        if(!_tag){
            num = _header.number - CHANGE_VALIDATORS_SIZE;
        }

        Validator memory v = _getCanVerifyValidator(num,_tag);

        require(v.headerHeight > 0, "validator load fail");

        require(v.headerHeight + CHANGE_VALIDATORS_SIZE >= _header.number, "check block height error");

        success = _checkCommittedAddress(v.validators, signer);

        require(success, "signer fail");

        bytes memory committedMsg = abi.encodePacked(hash, MSG_COMMIT);

        return (_checkCommitSeal(v, committedMsg, ext.committedSeal), ext);
    }


    function _getEndValidatorHeight()
    internal
    view
    returns (uint256)
    {
        Validator memory v = validators[validatorIdx];
        return (v.headerHeight / CHANGE_VALIDATORS_SIZE + 1) * CHANGE_VALIDATORS_SIZE;
    }

    function _getStartValidatorHeight()
    internal
    view
    returns (uint256)
    {
        uint idx = validatorIdx;
        uint start = validators[idx].headerHeight;
        for (uint i = 0; i < MAX_VALIDATORS_SIZE; i++) {
            if (idx == 0) {
                idx = MAX_VALIDATORS_SIZE - 1;
            } else {
                idx --;
            }
            Validator memory v = validators[idx];
            if (v.headerHeight != 0 && v.headerHeight < start) {
                start = v.headerHeight;
            } else {
                break;
            }
        }
        return start;

    }

    function _getValidatorIndex(uint _startHeight)
    internal
    pure
    returns (uint)
    {
        return (_startHeight / CHANGE_VALIDATORS_SIZE) % MAX_VALIDATORS_SIZE;
    }


    function _getNextValidatorIndex() internal view returns (uint){
        if (validatorIdx == MAX_VALIDATORS_SIZE - 1) {
            return 0;
        }
        return validatorIdx + 1;
    }


    function _getCanVerifyValidator(uint256 _height,bool _tag)
    public
    view
    returns (Validator memory v)
    {
        uint256 opochBlockHeight = ((_height / CHANGE_VALIDATORS_SIZE)) * CHANGE_VALIDATORS_SIZE;
        if(extendList[opochBlockHeight] > 0){
            uint256 verifyHeight = _getBlockHeightList(_height,_tag);
            if(opochBlockHeight == verifyHeight){
                uint256 idx = _getValidatorIndex(_height);
                v = validators[idx];
                return v;
            }else {
                return extendValidator[verifyHeight];
            }
        }else{
            uint256 idx = _getValidatorIndex(_height);
            v = validators[idx];
            return v;
        }
    }


    function _checkCommittedAddress(
        address[] memory _validators,
        address _address)
    internal
    pure
    returns (bool)
    {
        for (uint i = 0; i < _validators.length; i++) {
            if (_validators[i] == _address) return true;
        }
        return false;
    }

    /**
     * @dev Calculate the number of faulty nodes.
     * https://github.com/klaytn/klaytn/blob/841a8ad3b45e92f4ea378c1ee1f06cdb963afbac/consensus/istanbul/validator/default.go#L370
     *
     */
    function _getFaultyNodeNumber(uint256 _n) internal view returns(uint256 f){
        if(_n > committeeSize){
            _n = committeeSize;
        }
        if(_n % 3 == 0){
            f = _n / 3 - 1;
        }else{
            f = _n / 3;
        }
    }


    /**
     * @dev Check whether the CommitSeal is adequate.
     * https://github.com/klaytn/klaytn/blob/841a8ad3b45e92f4ea378c1ee1f06cdb963afbac/consensus/istanbul/backend/engine.go#L359
     *
     */
    function _checkCommitSeal(
        Validator memory _v,
        bytes memory _committedMsg,
        bytes[] memory _committedSeal)
    internal
    view
    returns (bool)
    {
        bytes32 msgHash = keccak256(_committedMsg);
        address[] memory miners = new address[](_v.validators.length);

        uint checkedCommittee = 0;
        for (uint i = 0; i < _committedSeal.length; i++) {
            address committee = verifyTool.recoverSigner(_committedSeal[i], msgHash);
            if (_checkCommittedAddress(_v.validators,committee) && !(verifyTool.isRepeat(miners,committee,i))) {
                checkedCommittee++;
            }
            miners[i] = committee;
        }
        return checkedCommittee > (_getFaultyNodeNumber(_v.validators.length)) * 2;
    }


    function _transferOwnership(address _newOwner)
    internal
    virtual
    override
    {
        super._transferOwnership(_newOwner);
        _changeAdmin(_newOwner);
    }


    /** UUPS *********************************************************/
    function _authorizeUpgrade(address)
    internal
    view
    onlyOwner
    override {}

    function getAdmin() external view returns (address){
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }

}