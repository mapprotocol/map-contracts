use ethabi::{Event, EventParam, Hash, Log, ParamType, RawLog};
use map_light_client::proof::{LogEntry, ReceiptProof};
use near_sdk::ext_contract;
use std::convert::From;

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

pub struct EVMEvent {
    pub address: Address,
    pub log: Log,
}

impl EVMEvent {
    pub fn from_log_entry_data(
        name: &str,
        params: EthEventParams,
        log_entry: &LogEntry,
    ) -> Option<EVMEvent> {
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
        let mcs_address = log_entry.address;
        let topics = log_entry.topics.iter().map(Hash::from).collect();

        let raw_log = RawLog {
            topics,
            data: log_entry.data.clone(),
        };

        let log = event.parse_log(raw_log).ok()?;
        Some(Self {
            address: mcs_address,
            log,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethabi::{param_type::Writer, Token};
    use map_light_client::{header::Hash as MapHash, traits::FromVec};
    use tiny_keccak::Keccak;

    impl EVMEvent {
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
                        name,
                        kind,
                        indexed,
                    })
                    .collect(),
                anonymous: false,
            };
            let params: Vec<ParamType> = event.inputs.iter().map(|p| p.kind.clone()).collect();
            let topics = indexes
                .into_iter()
                .map(|value| MapHash::from_vec(&value).unwrap())
                .collect();
            LogEntry {
                address: locker_address,
                topics: vec![vec![long_signature(&event.name, &params)], topics].concat(),
                data: ethabi::encode(&values),
            }
        }
    }

    fn long_signature(name: &str, params: &[ParamType]) -> MapHash {
        let mut result = [0u8; 32];
        fill_signature(name, params, &mut result);
        result
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
}
