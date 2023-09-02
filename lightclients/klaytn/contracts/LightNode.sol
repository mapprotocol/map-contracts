// SPDX-License-Identifier: MIT

pragma solidity 0.8.12;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@mapprotocol/protocol/contracts/interface/IMPTVerify.sol";
import "@mapprotocol/protocol/contracts/interface/ILightNode.sol";
import "@mapprotocol/protocol/contracts/lib/RLPReader.sol";

import "./interface/IVerifyTool.sol";
import "./interface/IKlaytn.sol";

contract LightNode is UUPSUpgradeable, Initializable, ILightNode, Ownable2Step {
    using RLPReader for bytes;
    using RLPReader for uint256;
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for RLPReader.Iterator;

    uint8   constant MSG_COMMIT = 2;
    uint256 constant ADDRESS_LENGTH = 20;
    uint256 constant MAX_EPOCH_SIZE = 2160;
    uint256 constant CHANGE_VALIDATORS_SIZE = 3600;
    uint256 constant RLP_INDEX = 3;
    bytes32 constant ADD_VALIDATOR = 0x9faa13f6fa6f531607d2fc3a8956aa591b138a5e2690cba6cd54f56e7b2324c8;
    bytes32 constant REMOVE_VALIDATOR = 0x3e9698b37f61d5135393cc4891dd22b1a42d2d350e5d561bcd6967bf75589818;

    IVerifyTool public verifyTool;
    IMPTVerify public mptVerifier;

    uint256 public firstEpochHeight;
    uint256 public lastEpochHeight;         // last epoch start height
    uint256 public lastCommitteeHeight;     // last validator set start height, the committee might start at the middle of one epoch

    mapping(uint256 => Validator) public extendValidator;   // extended Validators stored
    mapping(uint256 => uint256) public extendList;          // startHeight => nextStartHeight  last height stored

    Validator[MAX_EPOCH_SIZE] public validators;

    uint256 public committeeSize;

    struct Validator {
        address[] validators;
        uint256 headerHeight;
    }

    function initialize(
        address[] memory _validators,
        uint256 _headerHeight,
        address _verifyTool,
        address _mptVerifier
    )
    external
    initializer
    checkAddress(_verifyTool)
    checkAddress(_mptVerifier)
    checkMultipleAddress(_validators)
    {
        Validator memory _validator = Validator({
        validators : _validators,
        headerHeight : _headerHeight
        });
        firstEpochHeight = _headerHeight;
        lastEpochHeight = _headerHeight;
        uint256 validatorIdx = _getValidatorIndex(lastEpochHeight);
        validators[validatorIdx] = _validator;
        verifyTool = IVerifyTool(_verifyTool);
        mptVerifier = IMPTVerify(_mptVerifier);
        committeeSize = 31;

        _transferOwnership(tx.origin);
    }


    modifier checkAddress(address _address){
        require(_address != address(0), "Address is zero");
        _;
    }

    modifier checkMultipleAddress(address[] memory _addressArray){
        for (uint i = 0; i < _addressArray.length; i++) {
            require(_addressArray[i] != address(0), "Address have zero");
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
        IKlaytn.ReceiptProof memory receiptProof = abi.decode(_receiptProof, (IKlaytn.ReceiptProof));

        if (receiptProof.deriveSha == IKlaytn.DeriveShaOriginal.DeriveShaConcat) {
            IKlaytn.ReceiptProofConcat memory proof = abi.decode(receiptProof.proof, (IKlaytn.ReceiptProofConcat));
            IKlaytn.BlockHeader memory header = proof.header;

            (success, ) = checkBlockHeader(header,true);
            if (!success) {
                message = "DeriveShaConcat header verify failed";
                return(success, message, logs);
            }
            success = verifyTool.checkReceiptsConcat(proof.receipts, (bytes32)(header.receiptsRoot));
            if (success) {
                bytes memory bytesReceipt = proof.receipts[proof.logIndex];

                logs = bytesReceipt.toRlpItem().toList()[RLP_INDEX].toRlpBytes();

                message = "DeriveShaConcat mpt verify success";
                return(success, message, logs);
            }else{
                message = "DeriveShaConcat mpt verify failed";
                return(success, message, logs);
            }
        } else if (receiptProof.deriveSha == IKlaytn.DeriveShaOriginal.DeriveShaOriginal) {
            IKlaytn.ReceiptProofOriginal memory proof = abi.decode(receiptProof.proof, (IKlaytn.ReceiptProofOriginal));

            (success, ) = checkBlockHeader(proof.header, true);
            if (!success){
                message = "DeriveShaOriginal header verify failed";
                return(success, message, logs);
            }

            success = mptVerifier.verifyTrieProof(bytes32(proof.header.receiptsRoot), proof.keyIndex, proof.proof, proof.txReceipt);

            if (success) {
                message = "DeriveShaOriginal mpt verify success";

                logs = proof.txReceipt.toRlpItem().toList()[RLP_INDEX].toRlpBytes();

                return(success, message, logs);
            } else {
                message = "DeriveShaOriginal mpt verify failed";
                return(false, message, logs);
            }
        } else {
            message = "Klaytn verify failed";
            return(false, message, logs);
        }
    }

    function updateBlockHeader(bytes memory _blockHeaders)
    external
    override
    {
        IKlaytn.BlockHeader[] memory _headers = abi.decode(
            _blockHeaders, (IKlaytn.BlockHeader[]));

        require(_headers[0].number > lastEpochHeight, "Height error");

        if (_headers[0].number % CHANGE_VALIDATORS_SIZE > 0) {
            _updateBlockHeaderChange(_headers);
        } else {
            for (uint256 i = 0; i < _headers.length; i++) {
                require(_headers[i].number == lastEpochHeight + CHANGE_VALIDATORS_SIZE, "Height epoch error");
                IKlaytn.BlockHeader memory bh = _headers[i];
                (bool success, IKlaytn.ExtraData memory data) = checkBlockHeader(bh, false);
                require(success, "Header verify fail");

                uint256 validatorIdx = _getValidatorIndex(bh.number);
                Validator memory tempValidators = validators[validatorIdx];

                cleanValidator(tempValidators.headerHeight);

                Validator memory v = Validator({
                validators : data.validators,
                headerHeight : bh.number
                });
                validators[validatorIdx] = v;
                lastEpochHeight = bh.number;

                firstEpochHeight = firstEpochHeight + CHANGE_VALIDATORS_SIZE;

                emit UpdateBlockHeader(msg.sender, lastEpochHeight);
            }
        }

    }

    function updateLightClient(bytes memory _data) external override {
    }

    function headerHeight() external override view returns (uint256 height) {
        return lastEpochHeight;
    }

    function clientState() external override view returns(bytes memory) {
        return bytes("");
    }

    function finalizedState(bytes memory _data) external override view returns(bytes memory) {
        return bytes("");
    }

    function verifiableHeaderRange()
    external
    override
    view
    returns (uint256 start, uint256 end) {
        start = firstEpochHeight;
        end = (lastEpochHeight / CHANGE_VALIDATORS_SIZE + 1) * CHANGE_VALIDATORS_SIZE - 1;
        //return (_getStartValidatorHeight(), _getEndValidatorHeight());
    }


    function getBytes(IKlaytn.ReceiptProofOriginal memory _proof)
    external
    pure
    returns (bytes memory)
    {
        bytes memory proof = abi.encode(_proof);

        IKlaytn.ReceiptProof memory receiptProof = IKlaytn.ReceiptProof(proof, IKlaytn.DeriveShaOriginal.DeriveShaOriginal);

        return abi.encode(receiptProof);
    }

    function getHeadersBytes(IKlaytn.BlockHeader[] memory _blockHeaders)
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

    // remove all validator sets in the epoch
    function cleanValidator(uint256 epochHeight) internal {
        uint256 nextHeight;
        uint256 height = epochHeight;
        while (extendList[height] > 0) {
            nextHeight = extendList[height];

            delete extendValidator[nextHeight];
            delete extendList[height];

            height = nextHeight;
        }
    }

    function _updateBlockHeaderChange(IKlaytn.BlockHeader[] memory _blockHeaders)
    internal
    {
        IKlaytn.BlockHeader memory header0 = _blockHeaders[0];
        IKlaytn.BlockHeader memory header1 = _blockHeaders[1];
        require(header0.voteData.length > 0,"The extension update is not satisfied");
        require(header0.number + 1 == header1.number, "Synchronous height error");

        require(header0.number >= lastCommitteeHeight, "Update height0 error");

        require(header1.number < lastEpochHeight + CHANGE_VALIDATORS_SIZE, "Update height1 error");

        //TODO: check parent hash

        (bool success, IKlaytn.ExtraData memory header1Extra) = checkBlockHeader(header1, true);
        (bool headerTag0,) = checkBlockHeader(header0, true);
        require(success && headerTag0, "Header change verify fail");

        IKlaytn.Vote memory vote = verifyTool.decodeVote(_blockHeaders[0].voteData);
        require( vote.value.length % ADDRESS_LENGTH == 0, "Address error");
        address[] memory newValidators = verifyTool.bytesToAddressArray(vote.value);
        bool success1;
        if (keccak256(vote.key) == ADD_VALIDATOR) {
            for(uint256 i = 0; i < newValidators.length; i++) {
                success1  = _checkCommittedAddress(header1Extra.validators, newValidators[i]);
                require(success1,"ADD_VALIDATOR error");
            }
        } else if (keccak256(vote.key) == REMOVE_VALIDATOR) {
            for(uint256 i = 0; i < newValidators.length; i++) {
                success1  = _checkCommittedAddress(header1Extra.validators, newValidators[i]);
                require(!success1 ,"REMOVE_VALIDATOR error");
            }
        } else {
            require(false, "Not the expected instruction");
        }

        Validator memory v = Validator({
        validators : header1Extra.validators,
        headerHeight : header1.number
        });
        extendValidator[header1.number] = v;

        uint256 startHeight = _getLastCommitteeHeight(header1.number, true);
        extendList[startHeight] = header1.number;

        lastCommitteeHeight = header1.number;
        emit UpdateBlockHeader(msg.sender, lastCommitteeHeight);
    }

    /**
     * @dev Gets the last changed committee height of this epoch,
             return epoch height if there is no changed committee
     */
    function _getLastCommitteeHeight(uint256 _height, bool _tag)
    internal
    view
    returns(uint256 lastHeight)
    {
        uint256 epochBlockHeight = (_height / CHANGE_VALIDATORS_SIZE) * CHANGE_VALIDATORS_SIZE;

        if (extendList[epochBlockHeight] > 0) {
            if (!_tag) {
                _height = _height + CHANGE_VALIDATORS_SIZE;
            }

            if (_height > lastCommitteeHeight && lastCommitteeHeight > lastEpochHeight) {
                lastHeight = lastCommitteeHeight;
            } else {
                lastHeight = _getCommitteeStartHeight(epochBlockHeight, _height);
            }
        } else {
            lastHeight = epochBlockHeight;
        }
    }

    /**
     * @dev Gets the height of the correct extendList
     *
     */
    function _getCommitteeStartHeight(uint256 _height, uint256 _verifyHeight)
    internal
    view
    returns (uint256) {
        uint256 height = _height;
        while (extendList[height] > 0) {
            if (extendList[height] >= _verifyHeight) {
                return height;
            }
            height = extendList[height];
        }

        return 0;
    }


    function checkBlockHeader(IKlaytn.BlockHeader memory _header, bool _tag)
    internal
    view
    returns (bool, IKlaytn.ExtraData memory)
    {
        require(_header.number >= firstEpochHeight, "Out of verifiable range");

        bool success = verifyTool.checkHeaderParam(_header);

        require(success, "Header param error");

        (bytes32 hash, bytes32 signerHash, IKlaytn.ExtraData memory ext) = verifyTool.getBlockHashAndExtData(_header);

        address signer = verifyTool.recoverSigner(ext.seal, keccak256(abi.encodePacked(signerHash)));

        uint num = _header.number;

        if (!_tag) {
            num = _header.number - CHANGE_VALIDATORS_SIZE;
        }

        Validator memory v = _getCanVerifyValidator(num, _tag);

        require(v.headerHeight > 0, "Validator load fail");

        require(v.headerHeight + CHANGE_VALIDATORS_SIZE >= _header.number, "Check block height error");

        success = _checkCommittedAddress(v.validators, signer);

        require(success, "Signer fail");

        bytes memory committedMsg = abi.encodePacked(hash, MSG_COMMIT);

        return (_checkCommitSeal(v, committedMsg, ext.committedSeal), ext);
    }


    function _getValidatorIndex(uint _startHeight)
    internal
    pure
    returns (uint)
    {
        return (_startHeight / CHANGE_VALIDATORS_SIZE) % MAX_EPOCH_SIZE;
    }

    function _getCanVerifyValidator(uint256 _height, bool _tag)
    public
    view
    returns (Validator memory v)
    {
        uint256 epochBlockHeight = ((_height / CHANGE_VALIDATORS_SIZE)) * CHANGE_VALIDATORS_SIZE;
        if (extendList[epochBlockHeight] > 0) {
            uint256 verifyHeight = _getLastCommitteeHeight(_height, _tag);
            if (epochBlockHeight == verifyHeight) {
                uint256 idx = _getValidatorIndex(_height);
                v = validators[idx];
                return v;
            } else {
                return extendValidator[verifyHeight];
            }
        } else {
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
        if (_n % 3 == 0) {
            f = _n / 3 - 1;
        } else {
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
            if (_checkCommittedAddress(_v.validators, committee) && !(verifyTool.isRepeat(miners,committee,i))) {
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