MASTER_ACCOUNT="maplabs.testnet" # make sure the account is already created on NEAR blockchain

# mcs factory contract
MCS_FACTORY_NAME=mfac # the name of mcs factory contract to be created, the account ID will be $MCS_FACTORY_NAME.$MASTER_ACCOUNT

# multisig contract
MULTISIG_NAME="multisig" # the name of multisig contract to be created, the account ID will be $MULTISIG_NAME.$MCS_FACTORY_NAME.$MASTER_ACCOUNT
MEMBERS=(m0.maplabs.testnet m1.maplabs.testnet m2.maplabs.testnet)  # the multisig members list, make sure these accounts have been created on NEAR blockchain
CONFIRMS=2  # the multisig confirmation number to trigger the execution of the request
REQUEST_LOCK=3600 # request cooldown period in seconds (time before a request can be executed)

# mcs contract
MCS_NAME="mos2"  # the name of mcs contract to be created, the account ID will be $MCS_NAME.$MCS_FACTORY_NAME.$MASTER_ACCOUNT
MAP_MCS_ADDRESS="B6c1b689291532D11172Fb4C204bf13169EC0dCA"  # the mcs contract address on MAP relay chain
WNEAR_ACCOUNT="wrap.testnet"  # wrapped near contract account on NEAR blockchain
NEAR_CHAIN_ID=5566818579631833089  # NEAR blockchain ID
MAP_CHAIN_ID=212  # MAP blockchain ID
CLIENT_ACCOUNT="client.cfac2.maplabs.testnet" # the account ID of the map light client contract which has already been deployed

export MCS_FACTORY_ACCOUNT=$MCS_FACTORY_NAME.$MASTER_ACCOUNT
export MCS_ACCOUNT=$MCS_NAME.$MCS_FACTORY_ACCOUNT
export MULTISIG_ACCOUNT=$MULTISIG_NAME.$MCS_FACTORY_ACCOUNT
export MEMBERS
export MASTER_ACCOUNT