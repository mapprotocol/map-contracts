MASTER_ACCOUNT=maplabs.testnet # make sure the account is already created on NEAR blockchain
FACTORY_NAME=cfac2 # the name of map client factory contract to be created, the account ID will be $FACTORY_NAME.$MASTER_ACCOUNT
CLIENT_NAME=client # the name of MAP light client contract to be created, the account ID will be $CLIENT_NAME.$FACTORY_NAME
MAP_RPC_URL=https://testnet-rpc.maplabs.io  # the RPC url of MAP blockchain
EPOCH_ID=6  # get the information of this epoch id to initialize the MAP light client contract
OWNER=multisig.mfac.maplabs.testnet # the owner of the MAP light client, which is a multisig-timelock contract