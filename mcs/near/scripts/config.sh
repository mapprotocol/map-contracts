MASTER_ACCOUNT="map003.testnet" # make sure the account is already created on NEAR blockchain

# mcs factory contract
MCS_FACTORY_NAME=mfac # the name of mcs factory contract to be created, the account ID will be $MCS_FACTORY_NAME.$MASTER_ACCOUNT

# multisig contract
MULTISIG_NAME="multisig" # the name of multisig contract to be created, the account ID will be $MULTISIG_NAME.$MCS_FACTORY_NAME.$MASTER_ACCOUNT
MEMBERS=(member0.map003.testnet member1.map003.testnet member2.map003.testnet)  # the multisig members list, make sure these accounts have been created on NEAR blockchain
CONFIRMS=2  # the multisig confirmation number to trigger the execution of the request
REQUEST_LOCK=5 # request cooldown period in seconds (time before a request can be executed)

# mcs contract
MCS_NAME="mcs"  # the name of mcs contract to be created, the account ID will be $MCS_NAME.$MCS_FACTORY_NAME.$MASTER_ACCOUNT
MAP_MCS_ADDRESS="F579c89C22DAc92E9816C0b39856cA9281Bd7BE0"  # the mcs contract address on MAP relay chain
WNEAR_ACCOUNT="wrap.testnet"  # wrapped near contract account on NEAR blockchain
NEAR_CHAIN_ID=1313161555  # NEAR blockchain ID
CLIENT_ACCOUNT="client.cfac.map003.testnet" # the account ID of the map light client contract which has already been deployed

export MCS_FACTORY_ACCOUNT=$MCS_FACTORY_NAME.$MASTER_ACCOUNT
export MCS_ACCOUNT=$MCS_NAME.$MCS_FACTORY_ACCOUNT
export MULTISIG_ACCOUNT=$MULTISIG_NAME.$MCS_FACTORY_ACCOUNT
export MEMBERS
export MASTER_ACCOUNT