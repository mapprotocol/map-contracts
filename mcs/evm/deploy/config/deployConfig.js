//mcs is generally deployed on an evm-compatible chain

//mcs chain mapclients lightnode address
let mcsLightNodeAddress = "0x1eD5058d28fCD3ae7b9cfFD0B0B3282d939c4034";
//mcs chain weth address
let mcsWethAddress = "0xB59B98DF47432371A36A8F83fC7fd8371ec1300B";
//mcs chain mapToken address
let mcsMapTokenAddress = "0xb245609e5b2a0E52191Cba6314b47C73a0f9f023";


//mcsRelay is generally deployed on the MAP chain

//mcsRelay chain lightNodeManager address
//lightNodeManager contract is map-contracts/protocol/contrats/LightClientManager.sol
let relayLightNodeManagerAddress = "0xCDD415445ddBFeC30a7B7EF73E20b1d5fFc3Ae10";
//mcsRelay chain wmap address
let relayWmapAddress = "0xC38D963541E07e552258C014CB22e35f26Fe355B";
//mcsRelay chain mapToken address
let relayMapTokenAddress = "0xfC109d725a41fFA5E50001c0B464438efBC197f2";

//deploy mapToken use name
let mapTokenName = "map token";
//deploy mapToken use symbol
let mapTokenSymbol = "MAPT";
//deploy Maklu cross chain token use name
let mccTokenName = "Maklu cross chain token";
//deploy Maklu cross chain token use symbol
let mccTokenSymbol = "MCCT";


module.exports = {

    mcsLightNodeAddress,
    mcsWethAddress,
    mcsMapTokenAddress,
    relayLightNodeManagerAddress,
    relayMapTokenAddress,
    relayWmapAddress,
    mapTokenName,
    mapTokenSymbol,
    mccTokenName,
    mccTokenSymbol


}