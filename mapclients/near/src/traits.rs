use crate::types::errors::Kind;

// "Deafult" trait is implemented for a few selected fixed-array types. Taken we can't implement
// the trait outside of a crate, we created a new one that mimics the stdlib.
pub trait DefaultFrom {
    fn default() -> Self;
}

pub trait FromBytes {
    fn from_bytes(data: &[u8]) -> Result<&Self, Kind>;
}

pub trait ToRlp {
    fn to_rlp(&self) -> Vec<u8>;
}

pub trait FromRlp {
    fn from_rlp(bytes: &[u8]) -> Result<Self, Kind>
    where
        Self: std::marker::Sized;
}

pub trait StateConfig {
    /// Epoch size expressed in number of blocks
    fn epoch_size(&self) -> u64;

    /// Defines how far block timestamp can go in the future
    fn allowed_clock_skew(&self) -> u64;

    /// Whether to validate (BLS signature) epoch headers. It should always be set to true.
    fn verify_epoch_headers(&self) -> bool;

    /// Whether to validate (BLS signature) non epoch headers. Since non-epoch don't affect
    /// validator set, it's acceptable to disable validation
    fn verify_non_epoch_headers(&self) -> bool;

    /// Whether to verify headers time against current time. It's recommended to keep it true
    fn verify_header_timestamp(&self) -> bool;
}
