// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/governance/TimelockController.sol";

contract RulingSignTimeLock is TimelockController {

    uint constant public MAX_OWNER_COUNT = 50;

    event Confirmation(address indexed sender, uint indexed transactionId);
    event Revocation(address indexed sender, uint indexed transactionId);
    event Submission(uint indexed transactionId);
    event Execution(uint indexed transactionId);
    event ExecutionFailure(uint indexed transactionId);
    event Deposit(address indexed sender, uint value);
    event OwnerAddition(address indexed owner);
    event OwnerRemoval(address indexed owner);
    event RequirementChange(uint required);
    event SpentAny(address to, uint transfer);

    mapping(uint => Transaction) public transactions;
    mapping(uint => mapping(address => bool)) public confirmations;
    mapping(address => bool) public isProposers;
    address[] public proposers;
    uint public required;
    uint public transactionCount;


    struct Transaction {
        address destination;
        uint value;
        bytes data;
        uint transactionType;
        bool executed;
    }

    modifier onlyContract() {
        require(msg.sender == address(this), "only contract");
        _;
    }

    modifier onlyProposer(address _proposer) {
        require(hasRole(PROPOSER_ROLE, _proposer), "only PROPOSER_ROLE");
        _;
    }

    modifier notProposer(address _proposer) {
        require(!hasRole(PROPOSER_ROLE, _proposer), "only not PROPOSER_ROLE");
        _;
    }


    modifier transactionExists(uint transactionId) {
        require(transactions[transactionId].destination != address(0), "transactionId is zero");
        _;
    }

    modifier confirmed(uint transactionId, address owner) {
        require(confirmations[transactionId][owner], "sender not confirmations");
        _;
    }

    modifier notConfirmed(uint transactionId, address owner) {
        require(!confirmations[transactionId][owner], "sender is confirmations");
        _;
    }

    modifier notExecuted(uint transactionId) {
        require(!transactions[transactionId].executed, "is executed");
        _;
    }

    modifier notNull(address _address) {
        require(_address != address(0), "address is zero");
        _;
    }

    modifier validRequirement(uint ownerCount, uint _required) {
        require(ownerCount <= MAX_OWNER_COUNT
        && _required <= ownerCount
        && _required > 0
            && ownerCount > 0, "validRequirement error");
        _;
    }

    /*
     * Public functions
     */
    /// @dev Contract constructor sets initial owners and required number of confirmations.
    /// @param _proposers List of initial proposers.
    /// @param _required Number of required confirmations.
    constructor (address[] memory _proposers, uint _required, uint _executors)
    TimelockController(_executors, _proposers, _proposers)
    validRequirement(_proposers.length, _required){
        for (uint i = 0; i < _proposers.length; i++) {
            if (isProposers[_proposers[i]] || _proposers[i] == address(0)) {
                revert();
            }
            isProposers[_proposers[i]] = true;
        }
        proposers = _proposers;
        required = _required;
    }


    function schedule(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 predecessor,
        bytes32 salt,
        uint256 delay
    ) public override onlyRole(PROPOSER_ROLE) onlyContract {
        super.schedule(target, value, data, predecessor, salt, delay);
    }

    function scheduleBatch(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata payloads,
        bytes32 predecessor,
        bytes32 salt,
        uint256 delay
    ) public override onlyRole(PROPOSER_ROLE) {}

    function execute(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 predecessor,
        bytes32 salt
    ) public payable override  {
        super.execute(target,value,data,predecessor,salt);
    }

    function executeBatch(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata datas,
        bytes32 predecessor,
        bytes32 salt
    ) public payable override  {}


    /// @dev Allows to add a new owner. Transaction has to be sent by wallet.
    /// @param _proposer Address of new owner.
    function addProposer(address _proposer) public
    onlyContract()
    notProposer(_proposer)
    notNull(_proposer)
    validRequirement(proposers.length + 1, required) {
        isProposers[_proposer] = true;
        proposers.push(_proposer);
        emit OwnerAddition(_proposer);
    }

    /// @dev Allows to remove an owner. Transaction has to be sent by wallet.
    /// @param _proposer Address of owner.
    function removeProposer(address _proposer) public
    onlyContract()
    onlyProposer(_proposer)
    validRequirement(proposers.length - 1, required) {
        isProposers[_proposer] = false;
        for (uint i = 0; i < proposers.length - 1; i++)
            if (proposers[i] == _proposer) {
                proposers[i] = proposers[proposers.length - 1];
                break;
            }
        proposers.pop();
        if (required > proposers.length)
            changeRequirement(proposers.length);
        emit OwnerRemoval(_proposer);
    }

    /// @dev Allows to replace an owner with a new owner. Transaction has to be sent by wallet.
    /// @param _proposer Address of owner to be replaced.
    /// @param _newProposer Address of new owner.
    function replaceProposer(address _proposer, address _newProposer) public
    onlyContract()
    notProposer(_newProposer)
    onlyProposer(_proposer) {
        for (uint i = 0; i < proposers.length; i++)
            if (proposers[i] == _proposer) {
                proposers[i] = _newProposer;
                break;
            }
        isProposers[_proposer] = false;
        isProposers[_newProposer] = true;
        emit OwnerRemoval(_proposer);
        emit OwnerAddition(_newProposer);
    }

    /// @dev Allows to change the number of required confirmations. Transaction has to be sent by wallet.
    /// @param _required Number of required confirmations.
    function changeRequirement(uint _required) public
    onlyContract
    validRequirement(proposers.length, _required) {
        required = _required;
        emit RequirementChange(_required);
    }

    function submitTransactionCall(address destination, uint value, bytes memory data) public
    returns (uint transactionId){
        uint8 tType = 1;
        if (destination == address(this)) {
            tType = 0;
        }
        transactionId = addTransaction(destination, value, tType, data);
        confirmTransaction(transactionId);
    }

    /// @dev Allows an owner to confirm a transaction.
    /// @param transactionId Transaction ID.
    function confirmTransaction(uint transactionId) public
    onlyProposer(msg.sender)
    transactionExists(transactionId)
    notConfirmed(transactionId, msg.sender) {
        confirmations[transactionId][msg.sender] = true;
        emit Confirmation(msg.sender, transactionId);
        executeTransactionOrToTimeLock(transactionId);
    }

    /// @dev Allows an owner to revoke a confirmation for a transaction.
    /// @param transactionId Transaction ID.
    function revokeConfirmation(uint transactionId) public
    onlyProposer(msg.sender)
    confirmed(transactionId, msg.sender)
    notExecuted(transactionId) {
        confirmations[transactionId][msg.sender] = false;
        emit Revocation(msg.sender, transactionId);
    }



    /// @dev Allows anyone to execute a confirmed transaction.
    /// @param transactionId Transaction ID.
    function executeTransactionOrToTimeLock(uint transactionId) public
    notExecuted(transactionId) {
        if (isConfirmed(transactionId)) {
            Transaction storage transaction = transactions[transactionId];
            transaction.executed = true;
            bool success = false;
            if (transaction.transactionType == 0) {
                (success,) = transaction.destination.call{value : transaction.value}(transaction.data);
            } else {
                (success,) = address(this).call(abi.encodeWithSignature(
                        "schedule(address,uint256,bytes,bytes32,bytes32,uint256)",
                        transaction.destination,
                        transaction.value,
                        transaction.data,
                        bytes32(0),
                        bytes32(transactionId),
                        3 days
                    ));
            }
            if (success) {
                emit SpentAny(transaction.destination, transaction.value);
            } else {
                emit ExecutionFailure(transactionId);
                transaction.executed = false;
            }
        }
    }

    /// @dev Returns the confirmation status of a transaction.
    /// @param transactionId Transaction ID.
    /// @return Confirmation status.
    function isConfirmed(uint transactionId) public view returns (bool){
        uint count = 0;
        for (uint i = 0; i < proposers.length; i++) {
            if (confirmations[transactionId][proposers[i]])
                count += 1;
            if (count == required)
                return true;
        }
        return false;
    }

    /*
     * Internal functions
     */
    /// @dev Adds a new transaction to the transaction mapping, if transaction does not exist yet.
    /// @param destination Transaction target address.
    /// @param value Transaction ether value.
    /// @param data Transaction data payload.
    /// @param TType 0 is self contract 1 call
    // / @return Returns transaction ID.
    function addTransaction(address destination, uint value, uint8 TType, bytes memory data)
    internal notNull(destination) returns (uint transactionId){
        transactionId = transactionCount;
        transactions[transactionId] = Transaction({
        destination : destination,
        value : value,
        data : data,
        transactionType : TType,
        executed : false
        });
        transactionCount += 1;
        emit Submission(transactionId);
    }

    /*
     * Web3 call functions
     */
    /// @dev Returns number of confirmations of a transaction.
    /// @param transactionId Transaction ID.
    /// @return count Number of confirmations.
    function getConfirmationCount(uint transactionId) public view returns (uint count){
        for (uint i = 0; i < proposers.length; i++)
            if (confirmations[transactionId][proposers[i]])
                count += 1;
    }

    /// @dev Returns total number of transactions after filers are applied.
    /// @param pending Include pending transactions.
    /// @param executed Include executed transactions.
    /// @return count Total number of transactions after filters are applied.
    function getTransactionCount(bool pending, bool executed) public view returns (uint count){
        for (uint i = 0; i < transactionCount; i++)
            if (pending && !transactions[i].executed
            || executed && transactions[i].executed)
                count += 1;
    }

    /// @dev Returns list of owners.
    /// @return List of owner addresses.
    function getProposers() public view returns (address[] memory){
        return proposers;
    }

    /// @dev transactionId Returns array with owner addresses, which confirmed transaction.
    /// @param transactionId Transaction ID.
    /// @return _confirmations Returns array of owner addresses.
    function getConfirmations(uint transactionId) public view returns (address[] memory _confirmations){
        address[] memory confirmationsTemp = new address[](proposers.length);
        uint count = 0;
        uint i;
        for (i = 0; i < proposers.length; i++) {
            if (confirmations[transactionId][proposers[i]]) {
                confirmationsTemp[count] = proposers[i];
                count += 1;
            }
        }
        _confirmations = new address[](count);
        for (i = 0; i < count; i++) {
            _confirmations[i] = confirmationsTemp[i];
        }
    }

    /// @dev Returns list of transaction IDs in defined range.
    /// @param from Index start position of transaction array.
    /// @param to Index end position of transaction array.
    /// @param pending Include pending transactions.
    /// @param executed Include executed transactions.
    /// @return _transactionIds Returns array of transaction IDs.
    function getTransactionIds(uint from, uint to, bool pending, bool executed) public view returns (uint[] memory _transactionIds){
        uint[] memory transactionIdsTemp = new uint[](transactionCount);
        uint count = 0;
        uint i;
        for (i = from; i < to; i++)
            if (pending && !transactions[i].executed || executed && transactions[i].executed) {
                transactionIdsTemp[count] = i;
                count += 1;
            }
        _transactionIds = new uint[](count);
        for (i = 0; i < count; i++) {
            _transactionIds[i] = transactionIdsTemp[i];
        }

        return _transactionIds;
    }
}