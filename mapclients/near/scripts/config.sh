MASTER_ACCOUNT=map003.testnet # make sure the account is already created on NEAR blockchain
FACTORY_NAME=cfac # the name of map client factory contract to be created, the account ID will be $FACTORY_NAME.$MASTER_ACCOUNT
CLIENT_NAME=client # the name of MAP light client contract to be created, the account ID will be $CLIENT_NAME.$FACTORY_NAME
MAP_RPC_URL=http://3.0.19.66:7445  # the RPC url of MAP blockchain
EPOCH_ID=300  # get the information of this epoch id to initialize the MAP light client contract