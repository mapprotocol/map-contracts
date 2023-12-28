// SPDX-License-Identifier: MIT

pragma solidity 0.8.21;

import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./interface/ILightNode.sol";
import "./interface/IVerifyTool.sol";
import "./interface/IZKVerifyTool.sol";
import "./bls/BlsCode.sol";
import "./bls/BGLS.sol";

contract LightNode is UUPSUpgradeable, Initializable, ILightNode, BGLS {
    uint256 constant INPUT_MASK = (~uint256(0) >> 3);
    uint256 constant MAX_VALIDATORS = 128;
    uint256 constant MAX_VALIDATORS_INFO = MAX_VALIDATORS * 32 * 5;

    IVerifyTool public verifyTool;
    IZKVerifyTool public zkVerifier;
    BlsCode blsCode;
    uint256 public validatorsCount;
    uint256 public maxValidators;
    uint256 public epochSize;
    uint256 public headerHeight;
    address private pendingAdmin;
    uint256 public startHeight;
    ValidatorCommitment public validatorCommitment;
    mapping(uint256 => ValidatorCommitment) public validatorCommitments;
    mapping(uint256 => bytes32) private cachedReceiptRoot;

    struct ValidatorCommitment {
        bytes32 commitment;
        uint256 epoch;
    }

    event mapInitializeValidators(uint256 _threshold, G1[] _pairKeys, uint[] _weights, uint256 epoch);
    event mapInitializeValidatorsCommitment(bytes32 commitment, uint256 epoch);
    event MapUpdateValidators(bytes32 commitment, uint256 epoch);
    event ChangePendingAdmin(address indexed previousPending, address indexed newPending);
    event AdminTransferred(address indexed previous, address indexed newAdmin);
    event NewVerifyTool(address newVerifyTool);
    event NewZKVerifier(address zkVerifier);

    modifier onlyOwner() {
        require(msg.sender == _getAdmin(), "LightNode: only admin");
        _;
    }

    /** initialize  **********************************************************/
    function initialize(
        // {pubkey_i.x.ci, pubkey_i.x.cr, pubkey_i.y.ci, pubkey_i.y.cr, weight_i}
        bytes memory validatorsInfo,
        uint _validatorsCount,
        uint _epoch,
        uint _epochSize,
        address _verifyTool,
        address _zkVerifier
    ) external override initializer {
        require(_epoch > 0, "LightNode: initializing epoch error");
        _changeAdmin(tx.origin);
        maxValidators = 1728000 / _epochSize;
        headerHeight = (_epoch - 1) * _epochSize;
        startHeight = headerHeight;
        epochSize = _epochSize;
        validatorsCount = _validatorsCount;

        // compute validators' commitment
        require(validatorsInfo.length == MAX_VALIDATORS_INFO, "LightNode: init validators error");
        bytes32 commitment = sha256(validatorsInfo);
        validatorCommitment.commitment = commitment;
        validatorCommitment.epoch = _epoch;
        validatorCommitments[_epoch % maxValidators] = ValidatorCommitment({commitment: commitment, epoch: _epoch});

        verifyTool = IVerifyTool(_verifyTool);
        zkVerifier = IZKVerifyTool(_zkVerifier);
        blsCode = new BlsCode();
        g1 = G1(1, 2);
        g2 = G2(
            0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
            0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
            0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
            0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b
        );
        emit mapInitializeValidatorsCommitment(commitment, _epoch);
    }

    function computeCommitment(bytes memory inputs) public view returns (bytes32 commitment) {
        assembly {
            let result := mload(0x40)

            let success := staticcall(gas(), 0x02, add(inputs, 0x20), mload(inputs), result, 0x20)
            if iszero(success) {
                revert(0, 0)
            }
            commitment := mload(result)
        }
    }

    function getBytes(
        receiptProof memory _receiptProof,
        uint256[8] memory _zkProofs
    ) public pure returns (bytes memory) {
        return abi.encode(_receiptProof, _zkProofs);
    }

    function setVerifyTool(address _verifyTool) external onlyOwner {
        verifyTool = IVerifyTool(_verifyTool);
        emit NewVerifyTool(_verifyTool);
    }

    function setZKVerifier(address _zkVerifier) external onlyOwner {
        zkVerifier = IZKVerifyTool(_zkVerifier);
        emit NewZKVerifier(_zkVerifier);
    }

    function verifyProofData(
        bytes memory _receiptProofBytes
    ) external view override returns (bool success, string memory message, bytes memory logsHash) {
        (receiptProof memory _receiptProof, uint256[8] memory _zkProofs) = abi.decode(
            _receiptProofBytes,
            (receiptProof, uint256[8])
        );

        return _verifyProofData(_receiptProof, _zkProofs);
    }

    function _verifyProofData(
        receiptProof memory _receiptProof,
        uint256[8] memory _zkProofs
    ) internal view returns (bool success, string memory message, bytes memory logsHash) {
        (uint min, uint max) = verifiableHeaderRange();
        uint height = _receiptProof.header.number;
        if (height <= min || height >= max) {
            message = "LightNode: header height error";
            return (false, message, logsHash);
        }

        logsHash = verifyTool.decodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
        (success, message) = verifyTool.getVerifyTrieProof(
            bytes32(_receiptProof.header.receiptHash),
            _receiptProof.keyIndex,
            _receiptProof.proof,
            _receiptProof.txReceiptRlp.receiptRlp,
            _receiptProof.txReceiptRlp.receiptType
        );
        if (!success) {
            message = "LightNode: receipt mismatch";
            return (success, message, logsHash);
        }
        success = verifyHeaderSig(_receiptProof.header, _receiptProof.ist, _zkProofs);
        if (!success) {
            message = "LightNode: verifyHeaderSig fail";
            return (success, message, logsHash);
        }
        return (success, message, logsHash);
    }

    function verifyProofDataWithCache(
        bytes memory _receiptProofBytes
    ) external override returns (bool success, string memory message, bytes memory logsHash) {
        (receiptProof memory _receiptProof, uint256[8] memory _zkProofs) = abi.decode(
            _receiptProofBytes,
            (receiptProof, uint256[8])
        );

        logsHash = verifyTool.decodeTxReceipt(_receiptProof.txReceiptRlp.receiptRlp);
        if (cachedReceiptRoot[_receiptProof.header.number] != bytes32("")) {
            (success, message) = verifyTool.getVerifyTrieProof(
                cachedReceiptRoot[_receiptProof.header.number],
                _receiptProof.keyIndex,
                _receiptProof.proof,
                _receiptProof.txReceiptRlp.receiptRlp,
                _receiptProof.txReceiptRlp.receiptType
            );
            if (!success) {
                message = "Mpt verification failed";
                return (success, message, logsHash);
            }
        } else {
            (success, message, logsHash) = _verifyProofData(_receiptProof, _zkProofs);
            if (success) {
                cachedReceiptRoot[_receiptProof.header.number] = bytes32(_receiptProof.header.receiptHash);
            }
        }
    }

    function updateBlockHeader(
        bytes memory curValiditors,
        blockHeader memory bh,
        istanbulExtra memory ist,
        uint256[8] memory zkProofs
    ) external override {
        require(bh.number % epochSize == 0, "LightNode: header number is error");
        require(bh.number > headerHeight, "LightNode: header is have");
        if (startHeight == 0) {
            startHeight = headerHeight - epochSize;
        }

        uint256 currentEpoch = getEpochNumber(bh.number) + 1;

        uint256 idPre = getValidatorsIdPrev(currentEpoch);

        bytes32 preCommitment = validatorCommitments[idPre].commitment;

        require(sha256(curValiditors) == preCommitment, "LightNode: header Validitors error");

        bool success = verifyHeaderSig(bh, ist, zkProofs);
        require(success, "LightNode: checkSig error");

        bytes memory newValidators = getValidatorInfo(ist, curValiditors);

        bytes32 newCommitment = sha256(newValidators);

        headerHeight = bh.number;
        validatorCommitments[currentEpoch % maxValidators] = ValidatorCommitment({
            commitment: newCommitment,
            epoch: currentEpoch
        });

        emit UpdateBlockHeader(msg.sender, bh.number);
        emit MapUpdateValidators(newCommitment, currentEpoch);
    }

    function getValidatorInfo(istanbulExtra memory ist, bytes memory curValiditors) internal returns (bytes memory) {
        bytes memory newValidators;
        uint256 bitmap = ist.removeList;
        assembly {
            let N := sload(validatorsCount.slot)
            newValidators := mload(0x40)

            let newValidatorsLength := 0
            for {
                let i := 0
            } lt(i, N) {
                i := add(i, 1)
            } {
                let bit := and(shr(i, bitmap), 1)
                if iszero(bit) {
                    for {
                        let j := 0
                    } lt(j, 160) {
                        j := add(j, 32)
                    } {
                        mstore(
                            add(add(newValidators, 0x20), add(newValidatorsLength, j)),
                            mload(add(add(curValiditors, 0x20), add(mul(i, 160), j)))
                        )
                    }
                    newValidatorsLength := add(newValidatorsLength, 160)
                }
            }
            mstore(newValidators, newValidatorsLength)
            mstore(0x40, add(newValidators, add(newValidatorsLength, 0x20)))
        }

        uint256 len = ist.addedPubKey.length;
        for (uint256 i = 0; i < len; i++) {
            newValidators = abi.encodePacked(newValidators, abi.encodePacked(ist.addedPubKey[i], uint256(1)));
        }
        validatorsCount = newValidators.length / 160;

        newValidators = abi.encodePacked(newValidators, new bytes(MAX_VALIDATORS_INFO - newValidators.length));

        return newValidators;
    }

    function verifyHeaderSig(
        blockHeader memory _bh,
        istanbulExtra memory ist,
        uint256[8] memory zkProofs
    ) internal view returns (bool success) {
        bytes32 extraDataPre = bytes32(_bh.extraData);
        (bytes memory deleteAggBytes, bytes memory deleteSealAndAggBytes) = verifyTool.manageAgg(ist);
        deleteAggBytes = abi.encodePacked(extraDataPre, deleteAggBytes);
        deleteSealAndAggBytes = abi.encodePacked(extraDataPre, deleteSealAndAggBytes);

        (bytes memory deleteAggHeaderBytes, bytes memory deleteSealAndAggHeaderBytes) = verifyTool.encodeHeader(
            _bh,
            deleteAggBytes,
            deleteSealAndAggBytes
        );

        (success, ) = verifyTool.verifyHeader(_bh.coinbase, ist.seal, deleteSealAndAggHeaderBytes);
        if (!success) return success;

        uint256 epoch = getEpochNumber(_bh.number);

        bytes32 commitment = validatorCommitments[epoch % maxValidators].commitment;

        success = checkSig(ist.aggregatedSeal.round, commitment, deleteAggHeaderBytes, zkProofs);
        return success;
    }

    function verifiableHeaderRange() public view override returns (uint256, uint256) {
        uint start;
        if (headerHeight > maxValidators * epochSize) {
            start = headerHeight - (maxValidators * epochSize);
        }

        if (startHeight > 0 && startHeight > start) {
            start = startHeight;
        }
        return (start, headerHeight + epochSize);
    }

    function checkSig(
        uint256 _round,
        bytes32 _commitment,
        bytes memory _headerWithoutAgg,
        uint256[8] memory zkProofs
    ) internal view returns (bool) {
        bytes memory message = getPrepareCommittedSeal(_headerWithoutAgg, _round);

        uint256 t0 = hashToBase(message, 0x00, 0x01);
        uint256 t1 = hashToBase(message, 0x02, 0x03);
        bytes32 hashT = sha256(abi.encodePacked(t0, t1));

        bytes32 commitment = sha256(abi.encodePacked(_commitment, hashT));

        uint[] memory inputs = new uint[](1);
        inputs[0] = uint256(commitment) & INPUT_MASK;

        return zkVerifier.verifyProof(zkProofs, inputs);
    }

    function getValidatorsId(uint256 epoch) internal view returns (uint) {
        return epoch % maxValidators;
    }

    function getValidatorsIdPrev(uint256 epoch) internal view returns (uint) {
        uint256 id = getValidatorsId(epoch);
        if (id == 0) {
            return maxValidators - 1;
        } else {
            return id - 1;
        }
    }

    function getPrepareCommittedSeal(
        bytes memory _headerWithoutAgg,
        uint256 _round
    ) internal pure returns (bytes memory result) {
        bytes32 hash = keccak256(_headerWithoutAgg);
        if (_round == 0) {
            result = abi.encodePacked(hash, uint8(2));
        } else {
            result = abi.encodePacked(hash, getLengthInBytes(_round), uint8(2));
        }
    }

    function getLengthInBytes(uint256 num) internal pure returns (bytes memory) {
        require(num < 2 ** 24, "LightNode: num is too large");
        bytes memory result;
        if (num < 256) {
            result = abi.encodePacked(uint8(num));
        } else if (num < 65536) {
            result = abi.encodePacked(uint16(num));
        } else {
            result = abi.encodePacked(uint24(num));
        }
        return result;
    }

    function getEpochNumber(uint256 blockNumber) internal view returns (uint256) {
        if (blockNumber % epochSize == 0) {
            return blockNumber / epochSize;
        }
        return blockNumber / epochSize + 1;
    }

    /** UUPS *********************************************************/
    function _authorizeUpgrade(address) internal view override {
        require(msg.sender == _getAdmin(), "LightNode: only Admin can upgrade");
    }

    function changeAdmin() public {
        require(pendingAdmin == msg.sender, "LightNode: only pendingAdmin");
        emit AdminTransferred(_getAdmin(), pendingAdmin);
        _changeAdmin(pendingAdmin);
    }

    function getPendingAdmin() external view returns (address) {
        return pendingAdmin;
    }

    function setPendingAdmin(address pendingAdmin_) public onlyOwner {
        require(pendingAdmin_ != address(0), "LightNode: pendingAdmin is the zero address");
        emit ChangePendingAdmin(pendingAdmin, pendingAdmin_);
        pendingAdmin = pendingAdmin_;
    }

    function getAdmin() external view returns (address) {
        return _getAdmin();
    }

    function getImplementation() external view returns (address) {
        return _getImplementation();
    }
}
