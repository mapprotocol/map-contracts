# MAP Protocol Contracts

## MAP Protocol
MAP Protocol is the omnichain layer for Web3 with provably secure cross-chain communication built on Light-client and zk-SNARK technology. MAP provides cross-chain infrastructure to public chains and dApps by connecting both EVM with non-EVM chains. Developers can access a full suite of SDKs so their dApps can easily become omnichain applications.

## <a id="repo"></a>Repo Structure

The repository has the following packages (sub projects):

- [Management protocol](protocol) - MAP Protocol management conctracts on MAP Relay Chain
- [MAP Relay Chain light client](mapclients) - MAP Relay Chain light client implementation on all chains
  - [EVM chains](mapclients/eth) - MAP Relay Chain light client on EVM chains
  - [Near](mapclients/near) - map relay chain light client on Near Protocol
- [light clients on MAP](lightclients) - scripts for deploying and managing testnets
  - [BNB Smart Chain light client](lightclients/bsc) - BNB smart chain light client on MAP Relay Chain
  - [Near light client](lightclients/near) - Near Protocl light client on map relay chain
- [MAP Omnichain Service](mcs) - MAP omnichain Service reference implementation, mos contracts will be deplyed on every chain to achieve cross-chain interoperablity
  - [MOS on evm chains](mcs/evm) - mos on evm chains
  - [MOS on near](mcs/near)

## How to Install

1. Relay Chain Protocol

2. MAP Light Client

3. Light Clients on MAP

4. MOS Relay contracts
  fee, register, relay, proxy
  set fee

5. MOS contracts on other chains
  mcs, proxy
  relay register (chain type, address)
  setrelay


6. token register (map <-> chain)
  relay:
  a. create map token if needed
  b. set auth token(if needed)
  c. create map vault
  d. bind vault and token
  e. set fee
  f. relay token register (decimal, chain token)
  
  mos:
  a. mos set auth token
  b. mos register token

7. token update
  a. relay set fee
  b. relay token register (decimal, chain token)
  c. mos set auth token if neened
  d. mos register token

7. token deploy
   a. 