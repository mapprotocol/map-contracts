// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

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
        uint256 from,
        uint256 to,
        bytes calldata headers
    ) external;

    /**
     * @dev call through pre-compiled contract
     *
     */
    function txVerify(
        address router,
        address coin,
        uint256 srcChain,
        uint256 dstChain,
        bytes calldata txProve
    ) external returns (bool success, string memory message);
}

/**
 * @dev interface for the precompiled contract
 */
interface IHeaderStoreContract {
    function save(
        uint256 from,
        uint256 to,
        bytes memory headers
    ) external;
}
