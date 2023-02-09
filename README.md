# MAP Protocol Contracts

## MAP Protocol
MAP Protocol is the omnichain layer for Web3 with provably secure cross-chain communication built on Light-client and zk-SNARK technology. MAP provides cross-chain infrastructure to public chains and dApps by connecting both EVM with non-EVM chains. Developers can access a full suite of SDKs so their dApps can easily become omnichain applications.

## <a id="repo"></a>Repo Structure

The repository has the following packages (sub projects):

- [Management protocol](protocol) - MAP Protocol management conctracts on MAP Relay Chain
- [MAPO Relay Chain light client](mapclients) - MAPO Relay Chain light client implementation on all chains
  - [EVM chains](mapclients/eth) - MAPO Relay Chain light client on EVM chains
  - [Near](mapclients/near) - MAPO Relay Chain light client on Near Protocol
- [light clients on MAPO](lightclients) - All light client depolyed on MAPO Relay Chain
  - [BNB Smart Chain light client](lightclients/bsc) - BNB smart chain light client on MAPO Relay Chain
  - [Near light client](lightclients/near) - Near Protocl light client on MAPO Relay Chain
- [MAP Omnichain Service](mos) - MAP omnichain Service reference implementation, mos contracts will be deplyed on every chain to achieve cross-chain interoperablity
  - [MOS on evm chains](mos/evm) - mos on evm chains
  - [MOS on near](mos/near)
