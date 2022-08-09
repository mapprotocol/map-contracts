use std::convert::From;
use ethabi::{Event, EventParam, Hash, Log, ParamType, RawLog, Token};
use ethabi::param_type::Writer;
use near_sdk::ext_contract;
use tiny_keccak::Keccak;
use map_light_client::{
    proof::ReceiptProof,
    proof::LogEntry,
    header::Hash as MapHash,
    traits::FromVec
};

pub type Address = [u8; 20];

pub fn validate_eth_address(address: String) -> Address {
    let data = hex::decode(address).expect("address should be a valid hex string.");
    assert_eq!(data.len(), 20, "address should be 20 bytes long");
    let mut result = [0u8; 20];
    result.copy_from_slice(&data);
    result
}

#[ext_contract(ext_map_light_client)]
pub trait MapLightClient {
    fn verify_proof_data(&self, receipt_proof: ReceiptProof);
}

pub type EthEventParams = Vec<(String, ParamType, bool)>;

pub struct MapEvent {
    pub mcs_address: Address,
    pub log: Log,
}

impl MapEvent {
    pub fn from_log_entry_data(name: &str, params: EthEventParams, log_entry: &LogEntry) -> Option<MapEvent> {
        let event = Event {
            name: name.to_string(),
            inputs: params
                .into_iter()
                .map(|(name, kind, indexed)| EventParam {
                    name,
                    kind,
                    indexed,
                })
                .collect(),
            anonymous: false,
        };
        let mcs_address = log_entry.address.clone();
        let topics = log_entry
            .topics
            .iter()
            .map(|h| Hash::from(h))
            .collect();

        let raw_log = RawLog {
            topics,
            data: log_entry.data.clone(),
        };

        let log = event.parse_log(raw_log).ok()?;
        Some(Self {
            mcs_address,
            log,
        })
    }

    pub fn to_log_entry_data(
        name: &str,
        params: EthEventParams,
        locker_address: Address,
        indexes: Vec<Vec<u8>>,
        values: Vec<Token>,
    ) -> LogEntry {
        let event = Event {
            name: name.to_string(),
            inputs: params
                .into_iter()
                .map(|(name, kind, indexed)| EventParam {
                    name: name.to_string(),
                    kind,
                    indexed,
                })
                .collect(),
            anonymous: false,
        };
        let params: Vec<ParamType> = event.inputs.iter().map(|p| p.kind.clone()).collect();
        let topics = indexes.into_iter().map(|value| MapHash::from_vec(&value).unwrap().clone()).collect();
        LogEntry {
            address: locker_address.into(),
            topics: vec![vec![long_signature(&event.name, &params)], topics].concat(),
            data: ethabi::encode(&values),
        }
    }
}

fn long_signature(name: &str, params: &[ParamType]) -> MapHash {
    let mut result = [0u8; 32];
    fill_signature(name, params, &mut result);
    result.into()
}

fn fill_signature(name: &str, params: &[ParamType], result: &mut [u8]) {
    let types = params
        .iter()
        .map(Writer::write)
        .collect::<Vec<String>>()
        .join(",");

    let data: Vec<u8> = From::from(format!("{}({})", name, types).as_str());

    let mut sponge = Keccak::new_keccak256();
    sponge.update(&data);
    sponge.finalize(result);
}
