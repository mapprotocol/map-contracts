// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/Address.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";


    struct blockHeader {
        bytes parentHash;
        address coinbase;
        bytes root;
        bytes txHash;
        bytes receipHash;
        bytes bloom;
        uint256 number;
        uint256 gasLimit;
        uint256 gasUsed;
        uint256 time;
        bytes extraData;
        bytes mixDigest;
        bytes nonce;
        uint256 baseFee;
    }

    struct txParams {
        address From;
        address To;
        uint256 Value;
    }

    struct txProve{
        bytes keyIndex;
        bytes[] prove;
        bytes expectedValue;
    }

    struct txLogs{
        uint index;
        Log[] logs;
    }

    struct Log {
        address addr;
        bytes[] topics;
        bytes data;
    }



    struct proveData {
        blockHeader header;
        txLogs logs;
        txProve prove;
        txParams Tx;
    }

interface IRelayer {
    event Register(address indexed from, uint256 value);
    event Unregister(address indexed from, uint256 value);
    event Withdraw(address indexed from, uint256 value);
    event WorkerSet(
        address indexed sender,
        uint256 indexed chainId,
        bytes32 indexed bindAddress
    );

    /** view functions */
    function address2Bytes(address addr) external pure returns (bytes32);

    function bytes2Address(bytes32 b32) external pure returns (address);

    function relayers() external view returns (address[] memory);

    function length() external view returns (uint256);

    function relayerAmount(address _relayer) external view returns (uint256);

    function relayerWorker(address _relayer, uint256 chainId)
    external
    view
    returns (bytes32);

    /**
     * @dev  register as a relayer.
     * meantime a minimum staking is required.
     *
     */
    function register() external payable;

    /**
     * @dev set thte worker address on a current chainId, only relayer can call this function
     *
     */
    function bind(address _worker) external;

    /**
     * @dev set thte worker address on a specified chainId, only relayer can call this function
     *
     */
    function bindingWorker(uint256 _chainId, bytes32 _worker) external;

    /**
     * @dev set a list of chainId with the same worker address, only relayer can call this function
     *
     */
    function batchBindingSingleWorker(
        uint256[] calldata _chainIdList,
        bytes32 _worker
    ) external;

    /**
     * @dev set a list of chainId  with respective worker address, only relayer can call this function
     *
     */
    function batchBindingWorkers(
        uint256[] calldata _chainIdList,
        bytes32[] calldata _workerList
    ) external;

    /**
     * @dev unregister, only relayer can call this function
     *
     */
    function unregister() external;

    /**
     * @dev after unregister, user can withdraw the staked funds
     *
     */
    function withdraw() external;

    /**
     * @dev call through pre-compiled contract
     *
     */
    function currentNumberAndHash(uint256 chainID)
    external
    view
    returns (uint256 number, bytes memory hash);

    /**
     * @dev call through pre-compiled contract
     *
     */
    function save(
//        uint256 from,
//        uint256 to,
        bytes calldata headers
    ) external;

    /**
     * @dev call through pre-compiled contract
     *
     */
    function txVerify(
        address router,
        uint256 srcChain,
        uint256 dstChain,
        proveData memory _proveData
    ) external returns (bool success, string memory message);
}

/**
 * @dev interface for the precompiled contract
 */
//interface IHeaderStoreContract {
//    function save(
//        uint256 from,
//        uint256 to,
//        bytes memory headers
//    ) external;
//}

interface IPreCompiledHeaderStore {
    function currentNumberAndHash(uint256 chainID)
        external
        view
        returns (uint256 number, bytes memory hash);

    function save(
//        uint256 from,
//        uint256 to,
        bytes calldata headers
    ) external;
}

interface IPreCompiledTxVerify {
    function txVerify(
        address router,
        uint256 srcChain,
        uint256 dstChain,
        proveData memory _proveData
    ) external returns (bool success, string memory message);
}

/**
 * @title the Relayer contract
 * @notice this contract manage relayers; save headers to pre-compiled contracts and
 *   provide tx verify interface to bridge contract
 */
contract Relayer is IRelayer, Initializable, Ownable {

    using EnumerableSet for EnumerableSet.AddressSet;

//    IPreCompiledHeaderStore constant HeaderStore =
//        IPreCompiledHeaderStore(0x5FbDB2315678afecb367f032d93F642f64180aa3);
//    IPreCompiledTxVerify constant TxVerify =
//        IPreCompiledTxVerify(0x5FbDB2315678afecb367f032d93F642f64180aa3);

    IPreCompiledHeaderStore constant HeaderStore =
    IPreCompiledHeaderStore(0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512);
    IPreCompiledTxVerify constant TxVerify =
    IPreCompiledTxVerify(0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512);

//0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512

    // minStakeAmout to registered as relayer
    uint256 public minStakeAmount;

    // stores the addresses of relayers
    EnumerableSet.AddressSet private _relayers;

    struct RelayerInfo {
        uint256 amount;
        // chainID => worker
        mapping(uint256 => bytes32) worker;
        uint256[] chainIdList;
        /** more field to add */
    }
    // registered relayers
    mapping(address => RelayerInfo) private _relayerInfo;

    // worker => chainID => relayer
    // for compatibility with another chains worker address using bytes32 instead
    mapping(bytes32 => mapping(uint256 => address)) public bindRelayer;

    // address => the amount can be withdraw
    mapping(address => uint256) public refund;

    // address of bridge contract
    address private _bridge;

    modifier onlyBridge() {
        require(_bridge == msg.sender, "Relayer: caller is not bridge");
        _;
    }

    modifier onlyRelayer() {
        require(
            _relayers.contains(msg.sender),
            "Relayer: caller is not relayer"
        );
        _;
    }

    modifier onlyWorker() {
        // only current chainId counts
        uint256 _chainId;

        // solhint-disable-next-line no-inline-assembly
        assembly {
            _chainId := chainid()
        }

        bytes32 _worker = address2Bytes(msg.sender);
        require(
            bindRelayer[_worker][_chainId] != address(0),
            "Relayer: caller is not worker"
        );
        _;
    }

    /** initialize  **********************************************************/
    function initialize(uint256 _minStakeAmount) external initializer {
        minStakeAmount = _minStakeAmount;
        _transferOwnership(msg.sender);
    }

    constructor() {
    }

    /** pure and view functions **********************************************************/

    function address2Bytes(address addr)
        public
        pure
        override
        returns (bytes32)
    {
        // padding left
        return bytes32(uint256(uint160(addr)));
    }

    function bytes2Address(bytes32 b32)
        public
        pure
        override
        returns (address)
    {
        // retrive low 20bytes
        return address(uint160(uint256(b32)));
    }

    function relayers() external view override returns (address[] memory) {
        return _relayers.values();
    }

    function length() external view override returns (uint256) {
        return _relayers.length();
    }

    function relayerAmount(address _relayer)
        external
        view
        override
        returns (uint256)
    {
        return _relayerInfo[_relayer].amount;
    }

    function relayerWorker(address _relayer, uint256 chainId)
        external
        view
        override
        returns (bytes32)
    {
        return _relayerInfo[_relayer].worker[chainId];
    }

    /** user functions **********************************************************/

    /**
     * @dev  IRelayer.register
     */
    function register() external payable override {
        require(
            msg.value >= minStakeAmount,
            "Relayer: insufficient stake amount"
        );
        require(!_relayers.contains(msg.sender), "Relayer: already registered");

        _addRelayer(msg.sender, msg.value);

        emit Register(msg.sender, msg.value);
    }

    /**
     * @dev  IRelayer.bind for self chain convience
     */
    function bind(address _worker) external override onlyRelayer {
        bytes32 b32worker = address2Bytes(_worker);

        uint256 chainId;
        assembly {
            chainId := chainid()
        }

        require(
            bindRelayer[b32worker][chainId] == address(0),
            "Relayer: worker already binded"
        );

        _setBindAddress(msg.sender, b32worker, chainId);
        emit WorkerSet(msg.sender, chainId, b32worker);
    }

    /**
     * @dev  IRelayer.bindingWorker
     */
    function bindingWorker(uint256 _chainId, bytes32 _worker)
        external
        override
        onlyRelayer
    {
        require(
            bindRelayer[_worker][_chainId] == address(0),
            "Relayer: worker already binded"
        );

        _setBindAddress(msg.sender, _worker, _chainId);
        emit WorkerSet(msg.sender, _chainId, _worker);
    }

    /**
     * @dev  IRelayer.batchBindingSingleWorker
     */
    function batchBindingSingleWorker(
        uint256[] calldata _chainIdList,
        bytes32 _worker
    ) external override onlyRelayer {
        for (uint256 i = 0; i < _chainIdList.length; i++) {
            uint256 chainId = _chainIdList[i];
            require(
                bindRelayer[_worker][chainId] == address(0),
                "Relayer: worker already binded"
            );

            _setBindAddress(msg.sender, _worker, chainId);
            emit WorkerSet(msg.sender, chainId, _worker);
        }
    }

    /**
     * @dev  IRelayer.batchBindingWorker
     */
    function batchBindingWorkers(
        uint256[] calldata _chainIdList,
        bytes32[] calldata _workerList
    ) external override onlyRelayer {
        require(
            _chainIdList.length == _workerList.length,
            "Relayer: List length must be equal"
        );

        for (uint256 i = 0; i < _chainIdList.length; i++) {
            uint256 chainId = _chainIdList[i];
            bytes32 worker = _workerList[i];
            require(
                bindRelayer[worker][chainId] == address(0),
                "Relayer: worker already binded"
            );

            _setBindAddress(msg.sender, worker, chainId);
            emit WorkerSet(msg.sender, chainId, worker);
        }
    }

    /**
     * @dev  IRelayer.Unregister
     */
    function unregister() external override onlyRelayer {
        uint256 amount = _removeRelayer(msg.sender);
        refund[msg.sender] = amount;
        emit Unregister(msg.sender, amount);
    }

    /**
     * @dev  IRelayer.withdraw
     */
    function withdraw() external override {
        require(refund[msg.sender] > 0, "Relayer: zero refund");

        /** to avoid reentrancy vulnerabilities
         * the Checks-Effects-Interactions pattern
         * https://solidity.readthedocs.io/en/v0.5.11/security-considerations.html#use-the-checks-effects-interactions-pattern[checks-effects-interactions pattern].
         */
        uint256 amount = refund[msg.sender];
        refund[msg.sender] = 0;
        Address.sendValue(payable(msg.sender), amount);

        emit Withdraw(msg.sender, amount);
    }

    /** pre-compiled functions **********************************************************/

    function currentNumberAndHash(uint256 chainID)
        external
        view
        override
        returns (uint256 number, bytes memory hash)
    {
        (number, hash) = HeaderStore.currentNumberAndHash(chainID);
    }

    function save(
//        uint256 from,
//        uint256 to,
        bytes calldata headers
    ) external override onlyWorker {
        HeaderStore.save( headers);
    }

    function txVerify(
        address router,
        uint256 srcChain,
        uint256 dstChain,
        proveData memory _proveData
    )
        external
        override
        onlyBridge
        returns (bool success, string memory message)
    {
        (success, message) = TxVerify.txVerify(
            router,
            srcChain,
            dstChain,
                _proveData
        );
    }

    /** owner functions **********************************************************/
    function setMinStakeAmount(uint256 _newMinStakeAmount) external onlyOwner {
        minStakeAmount = _newMinStakeAmount;
    }

    function setBridgeAddr(address _bridgeAddr) external onlyOwner {
        _bridge = _bridgeAddr;
    }

    /** internal functions **********************************************************/

    function _addRelayer(address _relayer, uint256 _amount) internal {
        RelayerInfo storage ri = _relayerInfo[_relayer];
        ri.amount = _amount;

        _relayers.add(_relayer);
    }

    function _removeRelayer(address _relayer)
        internal
        returns (uint256 amount)
    {
        RelayerInfo storage ri = _relayerInfo[_relayer];
        amount = ri.amount;

        // remove all bind worker
        for (uint256 i = 0; i < ri.chainIdList.length; i++) {
            uint256 chainId = ri.chainIdList[i];
            delete bindRelayer[ri.worker[chainId]][chainId];
        }

        delete _relayerInfo[_relayer];

        _relayers.remove(_relayer);
    }

    function _setBindAddress(
        address _relayer,
        bytes32 _worker,
        uint256 _chainId
    ) internal {
        RelayerInfo storage ri = _relayerInfo[_relayer];

        if (ri.worker[_chainId] == bytes32(0)) {
            // new added
            ri.chainIdList.push(_chainId);
        }

        ri.worker[_chainId] = _worker;
        bindRelayer[_worker][_chainId] = _relayer;
    }
}
